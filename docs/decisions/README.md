# Decisions

This repository's architecture decision records, `ADR-NNNN-<slug>.md`,
numbered from 0001. "ADR-NNNN" anywhere in this repository means a file here
on `main`. The Rust workspace's own ADR-0001 to ADR-0005 are cited as
`rust-final:conventions/adr/ADR-NNNN`.

## The interim rule

Writ has no implementation yet, so this repository is governed as the
definition's D1.22 rules, the path Writ takes for its own ADRs:

- **Until Writ #7**, these files are authoritative, by review, under the
  interim rule of Writ ADR 0008. Review keeps ADR discipline: an accepted ADR
  is not edited, a contradicting proposal supersedes it by name and never
  diverges silently (ADR-0001 D3), and `main` is fast-forwarded only at an
  approved, checked head (ADR-0007 D4).
- **From Writ #7**, `writ import` reads these files once, with a baseline.
  From then on the embedded log is authoritative and the files are its
  rendering (ADR-0001 D2).
- **From Writ #8**, code lands through `writ land`.

Every package entry carries `Adr=`, naming the ADR here that admits it
(ADR-0004 D1). The build fails on one without it, from slice 0.

## The shape

Each ADR has an H1 `# ADR-NNNN: <title>` matching its file name, a
`**Status:**` line, `## Alternatives considered` and `## Consequences`, with
Context and a numbered decision set (D1, D2, …) between. `adr-shape` checks
it, and refuses any other file here but this one and the collected file.

## ADR-0001 to ADR-0012

These were accepted on 2026-10-30 and collected on 2026-11-03 into
[`collected-2026-11-03.md`](collected-2026-11-03.md), which is committed as it
was supplied. Each file is its section of the collected file, with the
heading promoted to `#` and its subheadings to `##`, and the text otherwise
byte for byte.

ADR-0001, ADR-0008, ADR-0009 and ADR-0011 cite rulings that Draft 4 of the
definition has since superseded or amended. Each carries, under its status
lines, the note the collected file's preamble gives for it, headed
"Note (2026-11-03, Draft 4)". Nothing else in them changes.

`adr-text` rebuilds every one of the twelve from the collected file, notes
included, and fails on any byte of difference.
