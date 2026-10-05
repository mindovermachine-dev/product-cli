//! A branch's findings as one set, read on either side of signing.
//!
//! The inbox pushes a signed branch when signing introduced no finding: the
//! findings after the write are a subset of those before it. A finding the
//! branch already carried does not hold back the holder's acceptances, and
//! is reported as remaining. Each finding is compared whole — stage, class,
//! subject and message — so one whose message changed reads as introduced,
//! and the comparison errs towards not pushing.

use std::collections::BTreeSet;
use std::path::Path;

/// One finding, rendered `[class] subject — message`.
pub type Findings = BTreeSet<String>;

/// Every finding `ledger verify --base <base> --export` reports in `dir`,
/// across the file gate, the graph stage, the export stage and the signers
/// stage.
pub fn read(dir: &Path, base: &str) -> Result<Findings, String> {
    let (stdout, stderr, code) = super::inbox::verify(dir, base, &["--json", "--export"])?;
    if code == 2 {
        return Err(format!("verify --base {base} could not run: {}", stderr.trim()));
    }
    let report: serde_json::Value = serde_json::from_str(&stdout).map_err(|e| format!("verify --json: {e}"))?;
    let mut out = Findings::new();
    for (stage, fallback) in [("findings", ""), ("graph", ""), ("export", "export"), ("signers", "signers")] {
        for f in report[stage].as_array().into_iter().flatten() {
            let class = f["class"].as_str().unwrap_or(fallback);
            let subject = f["subject"].as_str().unwrap_or_default();
            let message = f["message"].as_str().unwrap_or_default();
            out.insert(format!("[{class}] {subject} — {message}"));
        }
    }
    Ok(out)
}

/// The findings in `after` that `before` did not carry.
pub fn introduced<'a>(before: &'a Findings, after: &'a Findings) -> Vec<&'a String> {
    after.difference(before).collect()
}

/// The findings, one per indented line.
pub fn lines<'a>(findings: impl IntoIterator<Item = &'a String>) -> String {
    findings.into_iter().map(|f| format!("\n    {f}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_finding_the_write_added_is_introduced() {
        let before: Findings = ["[L001] dec:a — unallocated".to_string(), "[export] ns — stale".to_string()].into();
        let after: Findings = ["[L001] dec:a — unallocated".to_string(), "[L011] acc:b — bad signature".to_string()].into();
        assert_eq!(introduced(&before, &after), vec!["[L011] acc:b — bad signature"]);
        let fixed: Findings = ["[L001] dec:a — unallocated".to_string()].into();
        assert!(introduced(&before, &fixed).is_empty(), "a finding the write cleared is not introduced");
    }
}
