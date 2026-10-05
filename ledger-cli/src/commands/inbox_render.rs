//! The inbox on screen: per repository, per pull request or branch, newest
//! first, with each branch's index time and what stops a push to it.

use std::fmt::Write as _;

use ledger_core::inbox::index::{Branch, Index};
use ledger_core::inbox::list::Item;

use super::inbox::Sitting;

/// The whole list as text.
pub fn listing(s: &Sitting) -> String {
    let mut out = format!("inbox for {}\n", s.holder);
    for branch in &s.index.branches {
        let items: Vec<&Item> =
            s.listing.items.iter().filter(|i| i.repository == branch.repository && i.branch == branch.name).collect();
        let reviews: Vec<_> = s.reviews.iter().filter(|r| r.repository == branch.repository && r.branch == branch.name).collect();
        if items.is_empty() && reviews.is_empty() && branch.findings.is_empty() {
            continue;
        }
        out.push_str(&header(branch));
        for item in items {
            out.push_str(&entry(item));
        }
        for r in reviews {
            let _ = writeln!(
                out,
                "  needs re-acceptance ({}) {} — {} at {}{}",
                r.reason,
                r.acceptance,
                r.decision,
                short(&r.version),
                r.deadline.as_ref().map(|d| format!(", deadline {d}")).unwrap_or_default()
            );
        }
    }
    for u in &s.listing.unchecked {
        let _ = writeln!(
            out,
            "unchecked: `{}` in {} @ {} has no policy — its proposed decisions are in nobody's list",
            u.namespace, u.repository, u.branch
        );
    }
    let _ = writeln!(
        out,
        "\n{} proposed, {} awaiting re-acceptance across {} branch(es)",
        s.listing.items.len(),
        s.reviews.len(),
        s.index.branches.len()
    );
    out
}

fn header(b: &Branch) -> String {
    let label = match b.pr {
        Some(n) => format!("PR #{n} ({})", b.name),
        None => b.name.clone(),
    };
    let mut out = format!("\n{} — {label} @ {} (indexed {})\n", b.repository, short(&b.head), b.indexed_at.format("%Y-%m-%dT%H:%M:%SZ"));
    if let Some(why) = &b.push_unreachable {
        let _ = writeln!(out, "  remote unreachable for a push: {why}");
    }
    for f in &b.findings {
        let _ = writeln!(out, "  graph: [{}] {} — {}", f.class.code(), f.subject, f.message);
    }
    out
}

fn entry(i: &Item) -> String {
    let mut out = format!(
        "  {}{} [{} / {}] by {} ({})\n    {}\n    {} under {}\n",
        i.decision,
        i.key.as_ref().map(|k| format!(" ({k})")).unwrap_or_default(),
        i.namespace,
        i.set,
        i.proposer,
        i.identity_class,
        i.statement.lines().next().unwrap_or_default(),
        short(&i.version),
        i.grant
    );
    if let Some(r) = &i.rationale {
        let _ = writeln!(out, "    rationale: {r}");
    }
    if let Some(r) = &i.rule {
        let _ = writeln!(out, "    rule: {r}");
    }
    if !i.citing.is_empty() {
        let _ = writeln!(out, "    cites: {}", i.citing.join(", "));
    }
    if !i.grounds.is_empty() {
        let _ = writeln!(out, "    grounds: {}", i.grounds.join(", "));
    }
    for line in &i.diff {
        let _ = writeln!(out, "    {line}");
    }
    out
}

/// Each indexed branch, for `--json`.
pub fn branches_json(index: &Index) -> serde_json::Value {
    serde_json::Value::Array(
        index
            .branches
            .iter()
            .map(|b| {
                serde_json::json!({
                    "repository": b.repository, "branch": b.name, "head": b.head, "pr": b.pr,
                    "default": b.default, "base": b.base, "indexed_at": b.indexed_at.to_rfc3339(),
                    "push_unreachable": b.push_unreachable,
                    "graph": b.findings.iter().map(|f| format!("[{}] {} — {}", f.class.code(), f.subject, f.message)).collect::<Vec<_>>(),
                })
            })
            .collect(),
    )
}

fn short(hash: &str) -> &str {
    let h = hash.strip_prefix("sha256:").unwrap_or(hash);
    h.get(..12).unwrap_or(h)
}
