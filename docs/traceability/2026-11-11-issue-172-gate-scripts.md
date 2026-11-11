# AGENTS.md lists the gate scripts

> **Recorded contemporaneously**, by the session that did the work, under
> ADR-0007 D2, in the shape of Writ's `docs/traceability/` records (Writ ADR
> 0008). The maintainer's messages are verbatim. The transcript is held by the
> maintainer.

| | |
|---|---|
| **Issue** | [#172](https://github.com/mindovermachine-dev/product-cli/issues/172), opened by Emil (`Hafeok`) |
| **Plan** | [comment 6792713812](https://github.com/mindovermachine-dev/product-cli/issues/172#issuecomment-6792713812), which revises [comment 6792653737](https://github.com/mindovermachine-dev/product-cli/issues/172#issuecomment-6792653737) |
| **Approval** | [comment 6792726868](https://github.com/mindovermachine-dev/product-cli/issues/172#issuecomment-6792726868) |
| **Date** | 2026-11-11 |
| **Tool** | Claude Code, an issue session (ADR-0014): the routine in Emil's cloud environment, which holds the agent's signing key |
| **Routine prompt** | `eng/routines/issue-session.md`, version 1 |
| **Model** | not recorded (ADR-0007 D1) |
| **Session identifier** | `session_01NbWSYRLhTXXbd9ydccpqu3` |
| **Branch** | `claude/confident-franklin-2fu7qy`, from `main` at `78617cde1e9858520bb1a0c3ced2f7f698a32c06` |
| **Identity** | `Claude (Emil) <claude+emil@okkels-klein.dk>`, SSH-signed with key `SHA256:xDu5EyJd05ROum1g+8Aj1Up3MjE6RbHnI70/xNHljDY` |
| **Definition** | none read for a change: this is a documentation change to `AGENTS.md`, nothing structural |

## The prompts

The session's instructions were the routine's prompt,
`eng/routines/issue-session.md`, version 1. Its brief was issue #172, "add
gates to agent.md":

> AGENTS.md: list the gate scripts. Add a short list of the scripts in eng/gates/, one line each, saying what each checks, under the existing gates table. Change nothing else.

Three sessions ran on the issue, one per start:

1. **The `claude` label**, applied by `Hafeok` by hand. A session posted the
   plan, comment 6792653737, with two questions: which form the release
   descriptor takes, and the list's order and wording.
2. **Comment 6792695179**, typed by `Hafeok`:

   > @claude  1. a and 2 is fine approved

   It starts with `@claude` but not with `@claude approved`, so under the
   prompt's rule it asked for a revised plan. A session posted comment
   6792713812: the first plan with the two answers folded in and no question
   left.
3. **Comment 6792726868**, typed by `Hafeok`:

   > @claude approved

   It approves the newest plan above it, comment 6792713812. This session
   implemented that plan, and wrote this record.

## The start, checked against GitHub

Before anything else, this session checked the start's payload over REST:

- the repository is `mindovermachine-dev/product-cli`, and #172 is an issue
  with no `pull_request` field;
- comment 6792726868 is on issue #172, its author is `Hafeok`, it has no
  `performed_via_github_app`, and its body starts with `@claude approved`;
- the newest plan comment above it is 6792713812;
- no pull request, open or merged, carried `Approved plan: 6792713812`.

## What was read

- Issue #172 and its four comments.
- `AGENTS.md`, in full.
- The eight scripts in `eng/gates/`, for their header comments:
  `adr-shape.sh`, `adr-text.sh`, `claude-workflow.sh`, `commits.sh`,
  `no-toolchain.sh`, `selftest.sh`, `signatures.sh` and `smoke.sh`.
- `releases/v0.8.0-preview.3.yaml` and
  `docs/traceability/2026-11-06-issue-170-issue-sessions.md`, for the
  descriptor's and the record's shape.

## The approved plan

Comment 6792713812, in short:

- **`AGENTS.md`**: one addition in `## Gates`, directly under the table and
  above the paragraph that starts "`selftest` plants a failing case": a list
  of the eight scripts, alphabetical by file name, one line each. No other
  line of the file changes.
- **`releases/v0.8.0-preview.3.yaml`**: one line added to the summary's
  `AGENTS.md` bullet (Emil's answer 1, option a). No new version;
  `Directory.Build.props` stays at `0.8.0-preview.3`; `accepted-by` is left
  exactly as it is.
- **This record.**
- Three commits, the gate scripts run locally, and no `dotnet build` or
  `dotnet test`, since no built file changes.

## Commits

On `main` at `78617cd`, each signed as the agent identity, with `Refs #172`
and Emil's `Signed-off-by:`:

| SHA | Subject |
|---|---|
| `4d92b15724ba7c3a2abbc0098345314ae4c74f3f` | docs: AGENTS.md lists the gate scripts |
| `920123e1d8a6190b1f1df12175c6167fa8032418` | releases: the preview.3 descriptor names the gate-script list |
| this record's commit | docs(traceability): the #172 session record |

## Checks

Run locally, from the repository root, on `920123e`, and again on the head
with this record; all pass both times, and the pull request's description
carries the results on the head:

| Check | Result |
|---|---|
| `eng/gates/selftest.sh` | passes: "every check passes its clean case and fails every plant" |
| `eng/gates/commits.sh origin/main HEAD` | passes |
| `eng/gates/adr-shape.sh` | passes: 14 ADRs have the shape |
| `eng/gates/adr-text.sh` | passes: 12 ADRs equal their sections of the collected file |
| `eng/gates/no-toolchain.sh` | passes |
| `eng/gates/claude-workflow.sh` | passes |

- **No `dotnet build` or `dotnet test` locally.** No source, project or test
  file changed; `gates.yml` runs `build-test` and `aot` on the pull request.
- **`signatures`** runs on the pull request, since it reads GitHub's
  verification.

## Deviations from the plan

None.
