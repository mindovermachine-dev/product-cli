//! Adapters for the authority verbs: role, grant, identity, policy, (un)availability.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use ledger_core::author::{GrantArgs, InitNamespaceArgs, KeyArgs, PolicyArgs, RoleArgs, UnavailableArgs};
use ledger_core::authority::Authority;

use super::authority_enum::{GrantCmd, IdentityCmd, PolicyCmd, RoleCmd};
use super::common::{finish, open_author};
use super::{resolve_root, EXIT_OK};

fn parse_all<T: std::str::FromStr<Err = String>>(raw: &[String]) -> Result<Vec<T>, String> {
    raw.iter().map(|r| r.parse()).collect()
}

fn instant(raw: &str) -> Result<DateTime<Utc>, String> {
    raw.parse().map_err(|_| format!("`{raw}` is not an instant — expected RFC 3339, e.g. 2026-10-20T00:00:00Z"))
}

/// `ledger init --namespace`: scaffold, then put the namespace under policy.
pub fn init_namespace(root: PathBuf, args: InitNamespaceArgs) -> Result<i32, String> {
    let mut author = open_author(Some(root))?;
    finish(author.init_namespace(args))
}

pub fn role(root: Option<PathBuf>, cmd: RoleCmd) -> Result<i32, String> {
    let RoleCmd::Declare { id, namespace, may, title, notes } = cmd;
    let mut author = open_author(root)?;
    finish(author.declare_role(RoleArgs { namespace, id, title, may: parse_all(&may)?, notes }))
}

pub fn grant(root: Option<PathBuf>, cmd: GrantCmd) -> Result<i32, String> {
    match cmd {
        GrantCmd::Accept { grant } => finish(open_author(root)?.accept_grant(&grant.parse()?)),
        GrantCmd::New { role, to, scope, namespace, order, limits, supersedes, as_role } => {
            let args = GrantArgs {
                namespace,
                role,
                holder: to.parse()?,
                scope: scope.parse()?,
                order: order.parse()?,
                limits: parse_all(&limits)?,
                supersedes: supersedes.as_deref().map(str::parse).transpose()?,
            };
            let mut author = open_author(root)?;
            author.as_role = as_role;
            finish(author.grant(args))
        }
        GrantCmd::Revoke { grant, reason, as_role } => {
            let mut author = open_author(root)?;
            author.as_role = as_role;
            finish(author.revoke_grant(&grant.parse()?, reason))
        }
    }
}

/// Read an OpenSSH public key file: `<type> <base64> [comment]`.
fn key_file(path: &Path, namespace: String) -> Result<KeyArgs, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut parts = text.split_whitespace();
    match (parts.next(), parts.next()) {
        (Some(key_type), Some(key)) => Ok(KeyArgs { namespace, key_type: key_type.into(), key: key.into(), principal: None }),
        _ => Err(format!("{} is not an OpenSSH public key (`<type> <base64>`)", path.display())),
    }
}

pub fn identity(root: Option<PathBuf>, cmd: IdentityCmd) -> Result<i32, String> {
    if let IdentityCmd::Sync = cmd {
        let store = ledger_core::store::load(&resolve_root(root)?);
        let synced = ledger_core::authority::signers::write(&store)?;
        for line in synced.lines() {
            println!("{line}");
        }
        if synced.written.is_empty() && synced.removed.is_empty() {
            println!("no namespace binds a key — no {} to derive", ledger_core::authority::signers::FILE);
        }
        return Ok(EXIT_OK);
    }
    let mut author = open_author(root)?;
    match cmd {
        IdentityCmd::Add { namespace, key_file: path, principal } => {
            let mut args = key_file(&path, namespace)?;
            args.principal = principal.as_deref().map(str::parse).transpose()?;
            finish(author.identity_add(args))
        }
        IdentityCmd::Revoke { binding } => finish(author.identity_revoke(&binding.parse()?)),
        IdentityCmd::Rotate { binding, key_file: path } => {
            let closes = binding.parse()?;
            finish(author.identity_rotate(&closes, key_file(&path, String::new())?))
        }
        IdentityCmd::Sync => Ok(EXIT_OK),
    }
}

pub fn policy(root: Option<PathBuf>, cmd: PolicyCmd) -> Result<i32, String> {
    match cmd {
        PolicyCmd::Show { namespace } => show_policy(root, &namespace),
        PolicyCmd::Set { namespace, schemes, require_sk, accept_role, reaccept_within_days } => {
            let args = PolicyArgs {
                namespace,
                schemes: (!schemes.is_empty()).then(|| parse_all(&schemes)).transpose()?,
                require_sk,
                accept_role,
                reaccept_within_days: reaccept_within_days.map(|d| (d > 0).then_some(d)),
            };
            finish(open_author(root)?.policy_set(args))
        }
    }
}

fn show_policy(root: Option<PathBuf>, namespace: &str) -> Result<i32, String> {
    let store = ledger_core::store::load(&resolve_root(root)?);
    let auth = Authority::build(&store);
    let Some(p) = auth.policy(namespace) else {
        println!("namespace `{namespace}` has no policy — a pre-v2 namespace; nothing is role-checked");
        return Ok(EXIT_OK);
    };
    let schemes: Vec<&str> = p.schemes.iter().map(|s| s.as_str()).collect();
    println!("{} — policy in force for `{namespace}` (filed {} by {})", p.id, p.at, p.by);
    println!("  schemes: {}", schemes.join(", "));
    println!("  hardware keys (-sk) required: {}", if p.require_sk { "yes" } else { "no" });
    println!("  accept-decision role: {}", p.accept_role);
    match p.reaccept_within_days {
        Some(d) => println!("  re-acceptance deadline: {d} day(s) after a key closes"),
        None => println!("  re-acceptance deadline: none"),
    }
    println!("  hash: {}", p.hash);
    Ok(EXIT_OK)
}

pub fn unavailable(
    root: Option<PathBuf>,
    grant: &str,
    from: Option<&str>,
    until: Option<&str>,
    reason: Option<String>,
) -> Result<i32, String> {
    let mut author = open_author(root)?;
    let from = match from {
        Some(raw) => instant(raw)?,
        None => author.now,
    };
    let args = UnavailableArgs { grant: grant.parse()?, from, until: until.map(instant).transpose()?, reason };
    finish(author.unavailable(args))
}

pub fn available(root: Option<PathBuf>, interval: &str, at: Option<&str>) -> Result<i32, String> {
    let mut author = open_author(root)?;
    finish(author.available(&interval.parse()?, at.map(instant).transpose()?))
}
