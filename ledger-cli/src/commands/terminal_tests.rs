//! The closed lists: every verb on the surface is on exactly one side.

use std::collections::BTreeSet;

use clap::{CommandFactory, Parser};

use super::*;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

const DEC: &str = "dec:fixture.ledger/01K5M000000000000000000001";
const GRANT: &str = "grant:01K5M000000000000000000002";
const BINDING: &str = "key:01K5M000000000000000000003";

/// Refused unless stdin is a terminal: signing, and every authority record.
const TERMINAL: &[&[&str]] = &[
    &["accept", DEC],
    &["accept", "--set", "s", "--confirm", "sha256:00"],
    &["accept", "--group", "mechanical", "--confirm", "sha256:00"],
    &["accept", "--batch", "b.yml", "--confirm", "sha256:00"],
    &["accept", "--batch", "b.yml", "--repository", "r", "--branch", "b", "--confirm", "sha256:00"],
    &["available", "unav:01K5M000000000000000000004"],
    &["grant", "accept", GRANT],
    &["grant", "new", "acceptor", "--to", "a@x", "--scope", "*"],
    &["grant", "revoke", GRANT, "--reason", "r"],
    &["inbox", "accept", "--all", "--confirm", "sha256:00"],
    &["identity", "add", "--namespace", "n", "--key-file", "k.pub"],
    &["identity", "revoke", BINDING],
    &["identity", "rotate", BINDING, "--key-file", "k.pub"],
    &["init", "--namespace", "n", "--external-ref", "m"],
    &["policy", "set", "--namespace", "n"],
    &["revoke", "acc:01K5M000000000000000000005", "--reason", "r"],
    &["role", "declare", "r", "--may", "accept-decision"],
    &["unavailable", GRANT],
];

/// Scriptable: filing decisions, derived files, arbitration, and reads.
const SCRIPTABLE: &[&[&str]] = &[
    // The verbs that file decisions.
    &["add", "--set", "s", "--statement", "x"],
    &["allocate", DEC, "--store", "judgment"],
    &["declare", "--set", "s", "--tolerance-floor", "T1"],
    &["escape", DEC, "--exposure", "e", "--review-by", "2027-01-01"],
    &["revise", DEC, "--statement", "x"],
    &["supersede", DEC, "--by", DEC],
    // The derived-file verbs.
    &["identity", "sync"],
    &["merge", "--install"],
    &["merge-driver", "a", "b", "c", "p"],
    &["reindex"],
    // Scaffolding, arbitration (versions only), and the selection dry run.
    &["init"],
    &["merge", "--resolve"],
    &["accept", "--set", "s"],
    &["accept", "--group", "mechanical"],
    &["accept", "--batch", "b.yml"],
    &["inbox", "accept", "--all"],
    // Reads.
    &["blame", DEC],
    &["coverage"],
    &["diff", "HEAD"],
    &["export", "--format", "ntriples"],
    &["inbox", "list"],
    &["log"],
    &["merge", "main"],
    &["policy", "show", "--namespace", "n"],
    &["show", DEC],
    &["status"],
    &["verify"],
];

fn parse(args: &[&str]) -> Commands {
    let argv = std::iter::once("ledger").chain(args.iter().copied());
    match Cli::try_parse_from(argv) {
        Ok(cli) => cli.command,
        Err(e) => panic!("ledger {args:?} does not parse: {e}"),
    }
}

/// The subcommand path an invocation names: `grant new`, `verify`.
fn path_of(args: &[&str]) -> String {
    let mut cmd = Cli::command();
    let mut path = Vec::new();
    for arg in args {
        match cmd.find_subcommand(arg) {
            Some(sub) => {
                path.push(*arg);
                cmd = sub.clone();
            }
            None => break,
        }
    }
    path.join(" ")
}

/// Every leaf subcommand path clap knows, hidden ones included.
fn leaves(cmd: &clap::Command, prefix: &str, out: &mut BTreeSet<String>) {
    for sub in cmd.get_subcommands().filter(|s| s.get_name() != "help") {
        let path = if prefix.is_empty() { sub.get_name().to_string() } else { format!("{prefix} {}", sub.get_name()) };
        if sub.has_subcommands() {
            leaves(sub, &path, out);
        } else {
            out.insert(path);
        }
    }
}

#[test]
fn every_terminal_invocation_is_refused_without_a_terminal() {
    for args in TERMINAL {
        assert!(matches!(mode(&parse(args)), Mode::Terminal(_)), "ledger {args:?} must need a terminal");
    }
}

#[test]
fn every_scriptable_invocation_stays_scriptable() {
    for args in SCRIPTABLE {
        assert_eq!(mode(&parse(args)), Mode::Scriptable, "ledger {args:?} must stay scriptable");
    }
}

#[test]
fn every_verb_on_the_surface_is_classified() {
    let mut surface = BTreeSet::new();
    leaves(&Cli::command(), "", &mut surface);
    let classified: BTreeSet<String> = TERMINAL.iter().chain(SCRIPTABLE).map(|a| path_of(a)).collect();
    let unclassified: Vec<&String> = surface.difference(&classified).collect();
    assert!(unclassified.is_empty(), "classify these verbs in TERMINAL or SCRIPTABLE: {unclassified:?}");
    let stale: Vec<&String> = classified.difference(&surface).collect();
    assert!(stale.is_empty(), "these classified verbs no longer exist: {stale:?}");
}
