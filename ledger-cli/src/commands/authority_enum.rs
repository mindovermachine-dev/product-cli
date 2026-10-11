//! The authority subcommands, as clap reads them (spec v1.7).

use clap::Subcommand;

/// `ledger role …`
#[derive(Subcommand)]
pub enum RoleCmd {
    /// Declare a role file: what its grants may do (needs grant-role over *)
    Declare {
        /// Role id: lowercase alphanumerics, dashes, dots
        id: String,
        /// The namespace whose roles/ holds the file; inferred when the
        /// store holds one
        #[arg(long, value_name = "NS")]
        namespace: Option<String>,
        /// Capability, repeatable: accept-decision | sign-off-pattern |
        /// waive-invalidation | grant-role | revoke-grant |
        /// declare-unavailability | rotate-genesis
        #[arg(long = "may", value_name = "CAPABILITY", required = true)]
        may: Vec<String>,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        notes: Option<String>,
    },
}

/// `ledger grant …`
#[derive(Subcommand)]
pub enum GrantCmd {
    /// The holder accepts a grant: the grant is live only once accepted
    Accept {
        /// `grant:<ulid>`
        grant: String,
    },
    /// Grant a role over a scope (needs grant-role over that scope)
    New {
        /// The role granted
        role: String,
        /// The holder
        #[arg(long, value_name = "IDENTITY")]
        to: String,
        /// `*`, `ns:<namespace>`, `set:<set-id>` or `pattern:<id>`
        #[arg(long)]
        scope: String,
        /// `primary` or `fallback-N`
        #[arg(long, default_value = "primary")]
        order: String,
        /// Fallback limit, repeatable: no-grants | no-grant-revocations |
        /// no-genesis | no-role-edits
        #[arg(long = "limit", value_name = "LIMIT")]
        limits: Vec<String>,
        /// The grant this one replaces
        #[arg(long, value_name = "GRANT")]
        supersedes: Option<String>,
        /// The role this act is made under (D9): required only when more
        /// than one of your grants qualifies
        #[arg(long = "as", value_name = "ROLE")]
        as_role: Option<String>,
    },
    /// Revoke a grant, with the reason on record (needs revoke-grant)
    Revoke {
        grant: String,
        #[arg(long)]
        reason: String,
        /// The role this act is made under (D9): required only when more
        /// than one of your grants qualifies
        #[arg(long = "as", value_name = "ROLE")]
        as_role: Option<String>,
    },
}

/// `ledger identity …`
#[derive(Subcommand)]
pub enum IdentityCmd {
    /// Bind a public key in a namespace: your own, or (as the genesis
    /// holder) a principal's first key with --for (D7)
    Add {
        #[arg(long, value_name = "NS")]
        namespace: String,
        /// An OpenSSH public key file (`<type> <base64> [comment]`)
        #[arg(long, value_name = "PATH")]
        key_file: std::path::PathBuf,
        /// The principal whose first key this is (the genesis holder's act)
        #[arg(long = "for", value_name = "IDENTITY")]
        principal: Option<String>,
    },
    /// Close a key binding's window: yours, or any, as the genesis holder
    Revoke {
        /// `key:<ulid>` — the binding to close
        binding: String,
    },
    /// Close one of your bindings and bind a new key in the same act
    Rotate {
        /// `key:<ulid>` — the binding to close
        binding: String,
        #[arg(long, value_name = "PATH")]
        key_file: std::path::PathBuf,
    },
    /// Rewrite each namespace's allowed_signers from the log (after a merge)
    Sync,
}

/// `ledger policy …`
#[derive(Subcommand)]
pub enum PolicyCmd {
    /// File the namespace's next policy (the genesis holder's act)
    Set {
        #[arg(long, value_name = "NS")]
        namespace: String,
        /// Required signature scheme, repeatable: ssh | dsse | none
        #[arg(long = "scheme", value_name = "SCHEME")]
        schemes: Vec<String>,
        /// Require hardware-backed (`sk-`) keys: true | false
        #[arg(long, value_name = "BOOL")]
        require_sk: Option<bool>,
        /// The role whose grants carry accept-decision here
        #[arg(long, value_name = "ROLE")]
        accept_role: Option<String>,
        /// Days after a key closes to re-accept its acceptances (0 clears)
        #[arg(long, value_name = "DAYS")]
        reaccept_within_days: Option<u32>,
    },
    /// The policy in force for a namespace
    Show {
        #[arg(long, value_name = "NS")]
        namespace: String,
    },
}
