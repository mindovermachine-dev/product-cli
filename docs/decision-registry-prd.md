# Decision Registry — product requirements

Draft 1, 1 October 2026. Companion to the ledger CLI PRD and the Context& delivery platform PRD.

## 1. Problem

Acceptance is a human act; development is agentic and continuous. Today the two are coupled through the pull request: an agent files a decision, the PR goes red, and nothing moves until a holder opens that PR, reads the decision in the diff, and runs `ledger accept`. With several agents on several repositories, the holder is interrupted per PR and the agents idle per decision. The ledger CLI's `review` loop fixes the per-repository cost; it does not fix the coupling.

The registry decouples them. Agents propose and continue. Holders review on their own cadence, across all repositories and namespaces they hold, and acceptance flows back to the branches that need it.

## 2. What it is and is not

The registry is a **projection** over ledgers, not a store. Every ledger stays in its repository as files; the registry indexes proposed, accepted, rejected and superseded decisions across repositories, branches and namespaces, and provides the review loop over that index. It writes nothing except acceptance, rejection and revocation entities, and it writes those into the repositories as commits, exactly as the CLI would.

It is not a workflow engine, not a ticket system, and not a place where decisions can exist without a ledger.

One binary, three ways to run it, all on `ledger-core` and the same policy files:

- **`ledger inbox`** (open source): the CLI over a configured list of cloned repositories; signs with the holder's SSH key; pushes.
- **`ledger serve`** (open source): the same inbox as an HTTP service with a browser UI and a webhook receiver, for a team self-hosting its registry. Nothing is trusted differently from the CLI; see §7.
- **Hosted** (Context& ledger service): `ledger serve` with the Entra/Key Vault scheme and the GitHub App, run by Context& for customers (platform PRD §4.2).

## 3. Decision lifecycle

```
proposed ──review──▶ accepted ──▶ (superseded | revoked)
    │
    ├──▶ changes-requested ──▶ proposed'   (new version, same key)
    │
    └──▶ rejected
```

- **proposed**: filed by anyone, including an agent. Citable in development builds (CS0618 warning), not in release builds. Carries the proposer, the rationale, and the citing symbols at proposal time.
- **changes-requested**: a holder's response recorded on the version (`ledger:Review` entity: reviewer, verdict, comment, signed). The decision stays proposed; the next version answers the review.
- **rejected**: a signed review with verdict `reject`. The decision remains in the ledger as a record. Citations of a rejected decision are build errors on every configuration; the agent must take the design-change path.
- **accepted / superseded / revoked**: as in the ledger format v2.

`ledger:Review` is the one new entity the registry needs in the format. Verdicts: `accept` (which is the existing acceptance), `reject`, `changes-requested`. All signed under the same scheme as acceptances; same role requirement.

## 4. Decoupling rules

1. **Agents never wait.** An agent that files a decision opens or updates its PR and moves to its next task. Its instructions say so; the registry's existence is what makes that instruction safe.
2. **The PR check is the only coupling.** A PR with proposed decisions is red on the `ledger verify` check and nothing else. When acceptance commits land on the branch, the check re-runs and goes green. No agent involvement.
3. **Review results reach the agent as PR events.** The registry writes the review entity to the branch and posts a PR comment with the verdict and comment. A `changes-requested` or `reject` comment is a task for the agent, picked up like any review comment.
4. **Decisions can be accepted before any code cites them.** A holder files and accepts decisions directly in a set (from a design session, from a profile, from the catalogue). Agents are instructed to search accepted decisions before proposing; the generated `Ledger.<Ns>.<Set>.<Key>` types make the search a type search. Pre-accepted decisions are the main way review load drops over time.
5. **Merge policy is the repository's.** Default: a branch may not merge with proposed decisions. Allowed alternative, by accepted decision: merge with proposed decisions to a non-release branch; release builds still refuse. The registry supports both; the ruleset enforces.

## 5. Review loop

The inbox lists, per holder, every proposed decision in namespaces where the holder has `accept-decision`, grouped by repository and PR, newest first, with:

- key, statement, set, namespace; proposer and whether the proposer is an agent (identity class);
- rationale as filed; citing symbols with links to the lines on the branch;
- the rule that fired, when the decision came from an analyzer finding;
- the diff between this version and its predecessor if it is a revision;
- similar accepted decisions in the same namespace (text and key similarity), so duplicates are rejected with a pointer rather than accepted twice.

Actions: accept, request changes (with comment), reject (with comment), each on one or many selected items, one signing step per action batch. Target: twenty decisions in five minutes, as before, now across repositories.

Notification: a daily digest per holder (count by repository, oldest item age) and an immediate notice when a PR has been waiting past a threshold the holder set. Channels: email first; Teams and Slack webhooks later. No notifications per decision.

## 6. Agent protocol

What an agent's CLAUDE.md gains, verbatim:

> When a DD rule reports a finding, first search the accepted decisions of this namespace for one that covers it and cite that. If none does, decide whether the finding is a design error (fix the code) or a real decision (file it with `ledger new`, cite it, and continue). Do not wait for acceptance. Do not accept, revise or revoke decisions. If a review comment on your PR says `changes-requested` or `reject`, treat it as a required change.

The ledger refuses `accept`, `review`, `revoke` from agent identities (L006 today, policy roles in v2). The registry refuses them without an interactive holder session. There is no API path for an agent to move a decision past proposed.

## 7. Server mode and authentication

`ledger serve` adds an HTTP surface to the inbox: list and detail endpoints, envelope submission, a webhook receiver for pushes and PR events, and the browser UI. It writes to repositories through a GitHub App (contents write on PR branches, pull-request comments, checks) or an Azure DevOps service connection with the same scope.

Authentication rests on one principle: **writes are self-authenticating; sessions only guard reads.** An acceptance, review or revocation is a signed envelope. The server verifies it against the namespace policy exactly as `ledger verify` does, and the signature is the authority. No session, token or role on the server can produce an acceptance.

| Access | Open server | Hosted server |
|---|---|---|
| Writes (accept, review, revoke) | Signed envelope posted by the CLI; SSH scheme, `-sk` keys for hardware touch; principal checked against `allowed_signers` with validity windows | DSSE envelope signed by the holder's Key Vault key under Entra conditional access, posted by the browser inbox |
| Inbox reads (non-exported decisions, proposers, PR links) | Session bound to a `mailto:` principal: `ledger login <url>` signs a server nonce with `ssh-keygen -Y sign -n ledger-auth@<host>`; the server verifies against the union of its namespaces' `allowed_signers` and issues a short-lived token that also opens the browser UI. Alternatively any OIDC provider (GitHub, Entra, Keycloak) with the `email` claim mapped to the `mailto:` principal | Entra ID only, conditional access |
| Exported decisions | Anonymous | Anonymous |
| Agents | Read-only as a repository installation (GitHub App installation token or repo-scoped token), scoped to the namespaces of the repositories it is installed in. Policy roles name humans; an installation identity can never resolve to a holder role | Same |
| Server to repositories | GitHub App | GitHub App |

Rules:

- An envelope is accepted only when its principal equals the session's principal. A holder cannot relay another holder's envelope.
- The server never holds a signing key in the open scheme. Browser inbox actions produce a batch; `ledger accept --batch <id>` signs and pushes from where the key is. In the terminal, `ledger inbox` signs directly.
- A third scheme, `webauthn`, closes the browser gap for self-hosters: passkey assertions over the canonical acceptance hash, with credential public keys listed in policy the way SSH keys are. Later ADR; not R0.
- Tokens are short-lived, bound to the issuing server's host, and carry no role; roles are read from policy at each request.
- Every write and every session issuance is an audit event with principal, scheme, envelope hash and source.

## 8. Index and freshness

- Source: each repository's committed export (`ledger export`). Local (R0, as built): read from clones, on the default branch plus every remote-tracking branch whose committed export differs from the default branch's. The default branch is the configured one, else the remote's `HEAD` symref, else `main`. Git does not record whether a PR is open, so a PR number is a label only: it is read from `refs/pull/<n>/head` when the clone has fetched that ref and its head matches the branch, and grouping falls back to the branch otherwise. No forge API is called (§10.3). Hosted: fetched on push via the GitHub App, which does know which PRs are open.

  > **Amended 2026-10-05, as built in R0 (#92)**, on the principal's ruling of Session B close-out §3 question 3. The earlier text read "on every branch with an open PR, plus `main`".
- Index: an RDF dataset, one named graph per (repository, branch), rebuilt from exports; queries are SPARQL. Graph-stage shapes run per named graph, never over the union. R0 rebuilds the index on every invocation. Local: in-memory Oxigraph via the CLI's existing dependency; hosted: Varve when durable storage lands, Oxigraph until then.
- Freshness: the inbox shows the index time per branch; an action on a stale item re-reads the branch before signing and refuses if the version hash moved.

## 9. Phases

| Phase | Delivers | Depends on |
|---|---|---|
| R0 | `ledger inbox` over a list of local clones: list, accept, push. Reviews without the `Review` entity (accept only) | ledger v2 export, SSH acceptance |
| R1 | `ledger:Review` entity; reject and changes-requested; PR comments via `gh` | format bump |
| R1.5 | `ledger serve`: HTTP surface, SSH challenge login, envelope submission, browser triage with batch hand-off to the CLI, webhook index | R1 |
| R2 | Hosted inbox on the Context& ledger service: Entra/Key Vault scheme, GitHub App push-triggered index, daily digest | platform P2 |
| R3 | Similarity hints; pre-acceptance from the catalogue; Teams/Slack | |

## 10. Decisions to make

1. Whether `rejected` is a terminal state or a revision with verdict can reopen it. Proposal: terminal; a new decision with a new key is the way back.
2. Whether `changes-requested` reviews are signed or plain. Proposal: signed; they are statements by a holder and they appear in the audit trail.
3. Whether the local inbox requires clones or can work from the GitHub API. Proposal: clones; it keeps signing and pushing identical to single-repository use.
4. Merge with proposed decisions: decided, not allowed by default; enabling it is an accepted decision in the repository's namespace.
5. Which OIDC providers the open server supports in R1.5. Proposal: generic OIDC configuration only; GitHub and Entra as documented examples.

## 11. Acceptance criteria

- A holder with three repositories and twelve open agent PRs sees every proposed decision in one list, accepts or rejects them in one sitting, and every affected PR check turns green or receives a review comment without the holder opening a PR.
- An agent given the §6 protocol files a decision, continues, and later resolves a `changes-requested` comment by revising the decision, with no human prompt in between.
- No path exists by which an agent identity, a non-interactive session, or any server-side credential produces an acceptance, review or revocation; the test suite proves it for CLI and server, including an attempt to relay another holder's envelope.
- The index rebuilt from exports equals the index maintained incrementally (same equivalence test as every other projection).
