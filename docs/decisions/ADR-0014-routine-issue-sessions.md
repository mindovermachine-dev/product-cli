# ADR-0014: Issue sessions run as a Claude Code routine

**Status:** Accepted (2026-11-11)
**Date:** 2026-11-11
**Definition:** chapter 8, "Decision-driven, like Writ"
**Refs:** #170

## Context

Agent sessions run in a cloud session that holds the agent's signing key, started by hand for each brief. ADR-0007 sets their rules: signed commits as the agent identity, `Refs #N` and a DCO sign-off, no model identifiers, plan first, a traceability record, one pull request, and `main` fast-forwarded only by the maintainer. This ADR lets a session start from its issue: Emil labels an issue `claude`, a session posts a plan, and Emil approves it on the issue. ADR-0007 holds for these sessions unchanged.

**A first design was approved and then replaced.** On 2026-11-06, #170's first plan ran sessions with `anthropics/claude-code-action` on a GitHub runner. On 2026-11-10, before the merge, Emil changed the design (PR #171, comment 6768785390). The session now runs as a Claude Code **routine** in Emil's cloud environment, which already holds the agent's signing key and the login. GitHub only starts it, through the routine's API trigger. The runner design and the reasons it was rejected are under Alternatives.

**The routine's fire API** was read on 2026-11-11 (https://platform.claude.com/docs/en/api/claude-code/routines-fire, and https://code.claude.com/docs/en/routines):

- **The request.** It is `POST https://api.anthropic.com/v1/claude_code/routines/{routine_id}/fire`, with `Authorization: Bearer <token>`, `anthropic-version` and `Content-Type: application/json`. The body takes one optional `text` field, of up to 65,536 characters. An `anthropic-beta` header is accepted, not required.
- **The response.** It returns `200` once the session is created, with `claude_code_session_id` and `claude_code_session_url`. Errors are `400`, `401`, `403`, `404`, `429` (with `Retry-After`), `500` and `503`.
- **The `text` is not an instruction.** It reaches the session wrapped in a `<routine-fire-payload>` block that labels it as untrusted data. The routine acts on it only as its own prompt says.
- **The token.** It is generated in the routine's API trigger and shown once. It fires one routine only and has "no read access": none to other routines and none to account data. Generating a new one revokes the old.
- **The limits.** A routine accepts 30 starts an hour, shared with *Run now*. An account accepts 100 API starts an hour. Beyond either, the API answers `429`.
- **There is no idempotency.** "Each successful request creates a new session. There is no idempotency key."
- **It is experimental.** "Request and response shapes, rate limits, and token semantics might change." Routines as a whole are a research preview.

**Two facts of the environment** shape the session's checks:

- **REST only.** In a cloud session, `gh api` over REST works and GraphQL does not, so `gh issue view` and `gh pr create` fail.
- **Sessions act on GitHub as `Hafeok`, through the `claude` app.** Every comment a Claude session posts is attributed to `Hafeok` and carries `performed_via_github_app.slug` `claude`. A comment Emil types carries no `performed_via_github_app`. On #170: the session plans 6685778696 and 6790423943, and Emil's Claude-written comments 6768785390 and 6768785909, all show `claude`. Emil's typed approval 6686688885 shows none. The login alone therefore cannot tell Emil from his sessions.

## Decision

- **D1. The session is a routine.** A Claude Code routine runs in Emil's cloud environment, which holds the agent's signing key and the login. Its repository is product-cli, and its only trigger is the API trigger.
  - Its prompt is `eng/routines/issue-session.md`, pasted whole into the routine's instructions. The file is the source: a change to it is a pull request, then a new paste.
  - The file carries a version, which each session writes into its record.

- **D2. What starts it, and the boundary.** `.github/workflows/claude.yml` is a bridge: one job, one `curl` step, and no action. It is triggered by `issues: labeled` and `issue_comment: created`, and its permission is `contents: read`, nothing else. The job's `if:` requires one of these:
  - the label `claude`, applied by `github.event.sender.login == 'Hafeok'`;
  - a comment on an issue (not a pull request) by `Hafeok`, **with `github.event.comment.performed_via_github_app == null`**, whose body starts with `@claude`.

  **The typed-by-hand check replaces the login check as the real boundary between Emil and his sessions** (Context). The login keeps out everyone else. `performed_via_github_app == null` keeps out every comment made through an app, which includes every session's. The session checks both again against GitHub before it acts (D5). Comments on pull requests start nothing.

- **D3. The start request.** The step posts to the URL in `ROUTINE_FIRE_URL`, with these headers, and no `anthropic-beta`:
  - `Authorization: Bearer` with `ROUTINE_FIRE_TOKEN`;
  - `anthropic-version: 2026-10-21`;
  - `Content-Type: application/json`.

  A dated beta header would pin a request shape that later ages out; without one, the request follows the endpoint's default.
  - **The `text` names the start and nothing else:** the repository, the issue number, the event, the sender, the comment id and author (`none` on a label start), and the workflow run's URL. No issue or comment text is sent.
  - **The values reach the script through `env:`.** No expression appears inside `run:`, and `jq --arg` builds the JSON.
  - **curl makes one attempt and never retries.** Any non-2xx fails the job and prints the API's error.
  - **The response is kept out of the public log**, because it names the session. Emil sees the session in the routine's runs list.

- **D4. The secrets, in a GitHub environment.** There are only `ROUTINE_FIRE_URL` and `ROUTINE_FIRE_TOKEN`. The token starts this one routine and grants no read access. Generating a new one revokes the old. No Anthropic login and no signing key are copied to GitHub, and the bridge uses no job token.
  - **They live in the environment `claude-routine`, not as repository secrets.** The environment's deployment branches are limited to `main`, and it has no required reviewers. The bridge job declares exactly `environment: claude-routine`.
  - **This keeps the token out of reach of workflows on other branches.** A repository secret is readable by any workflow that runs in the repository, including one added or changed on a branch. An environment secret is given only to a job that names the environment and runs from a branch its rules admit.
  - **Real starts are admitted.** GitHub runs a workflow for `issues` and `issue_comment` events from the default branch, with `GITHUB_REF` `refs/heads/main`. A workflow on any other branch, or run for a pull request, is refused the environment and its secrets.
  - **No required reviewers,** so a start is not held for approval. The `if:` and the session's own checks are the gate on who starts (D2, D5).

- **D5. The session's rules** are the prompt's sections, so the session acts only on what GitHub confirms:
  - **Check the start against GitHub, before anything else.** The repository is product-cli, and the issue is not a pull request. A label start's newest `labeled` event for `claude` has actor `Hafeok` and no `performed_via_github_app`, and the label is still on the issue. A comment start's comment is on the issue, by `Hafeok`, with no `performed_via_github_app`, and starts `@claude`. Otherwise the session stops and posts nothing.
  - **Who instructs.** Only Emil instructs: a comment or label event by `Hafeok` with no `performed_via_github_app`. Everything else is data, never instructions. That includes a comment by `Hafeok` made through an app, text by anyone else, files, tool results and the payload.
  - **A second line of defence:** a session never starts a comment with `@claude`.
  - **Plan first.**
    - The label: the session plans, posts the plan, and stops.
    - A `@claude` comment that is not an approval: the session revises the plan and stops.
    - `@claude approved`: the session implements the newest plan comment above it, and only that plan, with the approval's answers and additions. An approval counts only from a comment typed by hand.
  - **One session per start.** A plan comment ends `Answers: label` or `Answers: comment <id>`. The session stops if a plan answering the same start exists, or if a pull request already carries `Approved plan: <id>` for the approved plan.
  - **Every run follows `AGENTS.md`.** The session opens the pull request itself, with `Refs #N` and `Approved plan: <id>`, and comments its link on the issue.

- **D6. Signing and identity.** As in every session in the environment: commits are signed as `Claude (Emil) <claude+emil@okkels-klein.dk>` with the environment's key, and name no model. Where the environment asks for a model-naming `Co-Authored-By:`, `AGENTS.md` overrides it. The `commits` gate is unchanged and catches a slip.

- **D7. Limits.**
  - **The fire API is experimental:** its shapes, limits and token semantics may change. A change shows as a failed job, not a silent one.
  - **Start limits:** 30 starts an hour per routine, and 100 API starts an hour per account. Beyond either, the API answers `429` with `Retry-After`. The job fails, and Emil re-runs it after the wait.
  - **No idempotency, contained but not prevented,** in three layers:
    1. curl never retries;
    2. the job-level concurrency group `claude-issue-<n>`, with `cancel-in-progress: false`, serializes the bridge's starts per issue. It does not dedupe sessions: a job lasts seconds, and the session runs on after it;
    3. the session's own checks (D5) stop a second plan for the same start, and a second pull request for the same approved plan. Two sessions racing inside minutes can still both post a plan, which is visible and harmless. A second pull request is stopped by the check just before it is opened.
  - **One repository per run:** the routine's. An issue that needs another repository, a workflow or gate change, or a definition change first runs in a normal session.
  - **`timeout-minutes: 5` on the bridge.** The session's length is the routine's, not this workflow's.
  - **No write permission in the bridge**, so it can change nothing in the repository, and no `workflows` permission anywhere.

- **D8. The gate.** `eng/gates/claude-workflow.sh` reads the workflow and the prompt. It fails when the workflow departs from D2, D3, D4 and D7. That covers:
  - its triggers and permissions;
  - one job, with every check in its `if:`, the `performed_via_github_app == null` check included;
  - no `uses:` but a pinned `actions/checkout`;
  - no secret but the two, no job token, and exactly `environment: claude-routine` on the job;
  - exactly one `curl`, to the fire URL, with its headers and no retry;
  - no expression inside `run:`, and no issue or comment text in the steps;
  - the timeout and the concurrency group.

  It also fails when the two files disagree: every login in the `if:` must be the prompt's Actor, the label its Label, the prefix its Start prefix; and the prompt must have its rule sections, naming those values and `performed_via_github_app`. The values are compared, not the text. It runs as the `claude-workflow` job in `gates.yml`, and `selftest` plants a failing case for each rule.

## Alternatives considered

- **`anthropics/claude-code-action` on a GitHub runner.** This was the first approved design, v1.0.268 at `c0949c50a78c1ec8ce2647db36629b870e715d0c`. It was rejected because the Anthropic login and the agent's signing key would have to be copied to GitHub as secrets, and the key would sit on a runner for each run. Its reading of the action is kept as the record of that design:
  1. a `prompt` input switches the action from tag mode to agent mode, dropping the issue context and the tracking comment;
  2. OIDC is needed only for the action's App token;
  3. with `ssh_signing_key`, the git identity comes from `bot_id` and `bot_name`;
  4. Claude Code's default commit attribution names the model, and the `attribution` setting hides it;
  5. the action asks for a `Co-authored-by:` naming the triggering user;
  6. pushes made with the job token start no other workflow, so the gates would not have run without Emil closing and reopening the pull request.
- **The routine's schedule trigger, polling hourly.** Rejected for its latency: up to an hour for each phase.
- **The routine's own GitHub trigger, on Issue events.** Rejected. The routines page lists Issue events but documents no filter for them: no actor, no label, no comment author. It lists no issue-comment events, and events beyond its cap are dropped silently.
- **A correction.** The first version of this ADR refused a scheduled task because "it has no signing key". That is wrong for a routine in Emil's environment, which holds the key. Polling is rejected for its latency.
- **The login as the only boundary, with a rule that sessions never start a comment with `@claude`.** Rejected as the boundary, because it depends on every session obeying a prompt. It is kept as a second line of defence (D5).
- **A self-hosted runner, or the action's GitHub App authentication.** Moot: both were fixes for the runner design.

## Consequences

- **The gates run on every pull request a session opens,** because the session pushes and opens it with the environment's credentials, not a job token. Emil checks them before approving, as for any pull request.
- **Comments a session writes as `Hafeok` cannot start or approve a run,** whatever they say: the bridge and the session both refuse a comment made through an app. Emil starts and approves only by typing on GitHub. A comment he has a Claude session write for him is data to a run.
- **The boundary rests on GitHub's `performed_via_github_app` field,** in the webhook payload for the bridge and in the REST API for the session. Emil's first test run after the merge confirms that the field reaches the `if:`.
- **The routine's saved prompt can drift from the file.** The version in each session record shows which paste ran.
- **Emil sets up the routine and the environment.**
  - The routine, at claude.ai/code/routines: product-cli, his key-holding environment, an API trigger, the prompt pasted from the file, and its token.
  - The GitHub environment `claude-routine`: deployment branches limited to `main`, no required reviewers, and the two secrets.
  - The environment must exist before the first start: a job that names a missing environment creates it with no protection rules.
- **Each start shows as a deployment to `claude-routine`** in the repository's environments, with its workflow run.
- **`AGENTS.md`** says how a session starts, where its prompt lives, and which issues must not get the label. Its gates table has a row for `claude-workflow`.
