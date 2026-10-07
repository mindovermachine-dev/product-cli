# The Decision Ledger Server-Client Protocol

5 October 2026 · Emil Klein

## Abstract

This document specifies how a decision server and its clients exchange reads and signed writes. Identity and read access are carried on OAuth 2.0; every write is a signed envelope that the server verifies under the Decision Ledger Protocol and commits unchanged. No token authorises a ledger act.

## Status of This Memo

This document is written in the form of an Internet-Draft. It has not been submitted to the IETF, is not an IETF document, and has no standing there or at W3C.

It is a work in progress dated 5 October 2026 and should be cited only as such. It was split from sections 10 to 12 of the Decision Ledger Protocol Editor's Draft, and its requirement identifiers are unchanged from that draft.

## 1. Introduction

A decision ledger lives as files in git repositories, and the Decision Ledger Protocol [LEDGER] defines when such a store is valid. This document defines the network side: how a holder sees what awaits them across repositories, and how a signed act travels from the holder's key to a commit.

One principle shapes everything here. Writes are self-authenticating, and sessions guard reads only. An acceptance, review or revocation is authorised by its signature under [LEDGER]; a token identifies who is asking and nothing more.

### 1.1. Requirements Language

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 [RFC2119] [RFC8174] when, and only when, they appear in all capitals, as shown here.

### 1.2. Requirement Identifiers

Each requirement carries an identifier of the form `SC-n.m` and the conformance classes it binds: **S** for a server, **C** for a client. Identifiers are stable and never reused. They do not follow this document's section numbers.

### 1.3. Terminology

The terms store, entity, version, act, holder, claim, finding and pin are used as defined in [LEDGER]. This document adds:

| Term | Definition |
| --- | --- |
| Server | A service that indexes repositories, issues tokens, serves reads and accepts envelopes. |
| Client | Software acting for one principal: it logs in, reads, signs locally and submits. |
| Principal | A `mailto:` identity as defined in [LEDGER]. |
| Installation | A non-human client bound to a set of repositories, such as an agent. |
| Interactive grant | A grant completed by a person: authorization code, device, or SSH signature. |
| Envelope | One signed entity with its sidecars, its target and the version hash it was made against. |
| Batch | An enumerated list of pending acts, pinned by a digest, handed from a browser session to a client. |
| Projection | State derived wholly from repositories and rebuildable from them. |

## 2. Protocol Overview

The protocol has two parties and one direction of dependency. It builds on [LEDGER], and [LEDGER] never refers to it.

- **SC-1.1** (S, C) This protocol depends on [LEDGER] and never the reverse. It never defines validity.
- **SC-1.2** (S) A server is a projection over repositories, not a store. Deleted and rebuilt from the repositories, it MUST yield the same index.
- **SC-1.3** (S) No endpoint, scope, claim or grant produces or authorises an acceptance, a review or a revocation.
- **SC-1.4** (S) A server accepts a write only if it is already valid under [LEDGER].

### 2.1. Roles

| OAuth 2.0 role | Played by |
| --- | --- |
| Authorization server | The decision server |
| Resource server | The same decision server |
| Client | A command-line client, a browser session or an installation |
| Resource owner | The principal |

### 2.2. Out of Scope

The following are implementation matters and are not specified:

- how a server learns of pushes and pull requests;
- how a server stores its index;
- how a server authenticates to a repository host and pushes commits;
- how a hosted server obtains a signature on a holder's behalf (but see Section 8.6).

## 3. Authentication and Tokens

Authentication is a profile of OAuth 2.0 [RFC6749] in which the decision server is the only issuer of the tokens it accepts.

### 3.1. Profile

- **SC-2.1** (S) The server is both resource server and authorization server. It is the only issuer of the tokens it accepts.
- **SC-2.2** (S, C) The profile follows [RFC9700]. PKCE [RFC7636] is REQUIRED. The implicit grant and the resource owner password credentials grant MUST NOT be used.
- **SC-2.3** (S) An external identity provider is an identity source only. The server completes the OpenID Connect login [OIDC-CORE], maps the verified `email` claim to a principal, and issues its own token.
- **SC-2.4** (S) The server MUST publish authorization server metadata [RFC8414], so that a client configures itself from the server's URL.

### 3.2. Grants

| Client | Grant | Token subject |
| --- | --- | --- |
| Browser session | Authorization code with PKCE | The principal |
| Client with an identity provider | Authorization code with PKCE on a loopback redirect [RFC8252], or the device grant [RFC8628] where there is no browser | The principal |
| Client with an SSH key | The SSH signature grant of Section 3.3 | The principal |
| Installation | Client credentials | The client identifier, never a principal |

### 3.3. The SSH Signature Grant

This is an extension grant in the sense of Section 4.5 of [RFC6749]. It lets a self-hosted server authenticate holders with the keys they already bind in [LEDGER], with no identity provider.

1. The client requests a nonce from the server's nonce endpoint.
2. The client signs the nonce as an SSH signature [SSHSIG] under the namespace string `ledger-auth@<host>`, where `<host>` is the server's host.
3. The client sends a token request carrying the grant type, the principal, the nonce and the signature.
4. The server checks that it issued the nonce and that the nonce is unused and unexpired.
5. The server verifies the signature for that principal against the key bindings of the namespaces it indexes, with their validity windows, at the current time.
6. The server issues a token response as in Section 5.1 of [RFC6749], or an `invalid_grant` error as in Section 5.2.

```
POST /token HTTP/1.1
Host: ledger.example
Content-Type: application/x-www-form-urlencoded

grant_type=<grant type URI, to be assigned>
&principal=mailto%3Aholder%40example.org
&nonce=<nonce>
&signature=<SSH signature>
```

- **SC-2.5** (S) The server verifies the signature against the key bindings of the namespaces it indexes, with their validity windows. It does not use `allowed_signers`, which accepts acceptance signatures only.
- **SC-2.6** (S) A nonce is single-use and short-lived.

The parameter names, the grant type URI and the nonce endpoint are provisional (Appendix A). The signing namespace `ledger-auth@<host>` differs from the acceptance namespace of [LEDGER], so a login signature can never be replayed as an acceptance.

### 3.4. Tokens

- **SC-2.7** (S) A token's audience is the issuing server's host. A token from one server MUST be refused by every other.
- **SC-2.8** (S) A token is short-lived and carries no role. Roles are read from namespace policy at each request.
- **SC-2.9** (S) Scopes cover reading and the transport permission to submit an envelope. No scope confers a ledger act.
- **SC-2.10** (S) An installation's token is read-only and is scoped to the namespaces of the repositories it is installed in.

## 4. Reads

Reads are either anonymous or guarded by a token. Nothing a read returns is authoritative: every item can be checked against its hash or signature under [LEDGER].

| Read | Access | Returns |
| --- | --- | --- |
| Exported decision | Anonymous | The decision's triples, by lineage, decision or version hash. |
| Version file | Anonymous | The file of an exported version by its hash, for holding a basis closure. |
| Version status | Anonymous | The acceptances and revocations of one version hash. |
| Inbox list | Token whose subject is a principal | Proposed decisions in namespaces where the principal holds the class's claim, grouped by repository and pull request. |
| Inbox detail | Token whose subject is a principal | One proposed version with its bases, their trust status and its predecessor. |
| Non-exported decision | Token | As namespace policy allows for the token's subject or installation. |

- **SC-3.1** (S) A non-exported decision MUST NOT be served anonymously.
- **SC-3.2** (S) Every listed item states the time its branch was indexed.

Resource paths and representations are not yet specified (Appendix A).

## 5. Writes

A write is one signed envelope. The server checks it exactly as a verifier would under [LEDGER], then commits it unchanged.

### 5.1. The Envelope

An envelope carries one entity's payload, its signature sidecars, the target repository and branch, and the version hash the act was made against.

### 5.2. Submission

```
 Client                  Server             Repository
   |                       |                     |
   | (1) sign locally      |                     |
   |                       |                     |
   | (2) envelope + token  |                     |
   |---------------------->|                     |
   |                       | (3) subject check   |
   |                       | (4) re-read branch  |
   |                       |-------------------->|
   |                       | (5) verify [LEDGER] |
   |                       | (6) commit as is    |
   |                       |-------------------->|
   |                       | (7) audit event     |
   | (8) result            |                     |
   |<----------------------|                     |

            Figure 1: Envelope Submission
```

1. The client builds the entity and signs its canonical bytes locally.
2. The client sends the envelope with its token.
3. The server checks that the token's subject is a principal from an interactive grant, and that it equals the envelope's principal.
4. The server re-reads the target branch and refuses if the version hash has moved.
5. The server verifies the entity under [LEDGER], as a verifier.
6. The server writes a commit containing exactly the entity file and its sidecars, and regenerates the derived files.
7. The server records an audit event.
8. The server returns the result, or a problem as in Section 5.4.

- **SC-3.3** (S) A holder cannot relay another holder's envelope. Step 3 refuses it.
- **SC-3.4** (S) A client-credentials token can never pass step 3, so an installation cannot submit an envelope.
- **SC-3.5** (S) The server MUST NOT alter a payload. Under the `ssh` scheme the server MUST NOT hold a holder's signing key.
- **SC-3.6** (S) A refusal changes nothing in any repository.

### 5.3. Batch Hand-off

- **SC-3.7** (S, C) A batch is the batch selection file of [LEDGER] (its Section 10.1): an enumerated list of rows pinned by a manifest digest under the canonical form of [LEDGER].
- **SC-3.8** (C) A client signs exactly the listed rows in one signing step and refuses on any drift: a moved version hash or a missing row.
- **SC-3.9** (S) A browser session MAY produce a batch. Only a client holding the key turns it into envelopes.

### 5.4. Errors

- **SC-3.10** (S) Errors are returned as problem details [RFC9457], each with a stable type.

| Condition | Step |
| --- | --- |
| Token subject is not a principal from an interactive grant | 3 |
| Token subject differs from the envelope's principal | 3 |
| Version hash moved on the target branch | 4 |
| Entity fails verification, with its findings | 5 |

### 5.5. Audit

- **SC-3.11** (S) Every write and every token issuance is an audit event with principal, scheme, envelope hash and source.

## 6. Network of Servers

Servers form a network without trusting each other. Trust runs between namespaces through pins, and every piece of data a server relays verifies itself under [LEDGER].

- **SC-4.1** (S) A namespace is a data domain on one server. Two servers MAY use the same namespace name for unrelated namespaces.
- **SC-4.2** (S) No token is shared or federated between servers. Each server issues and accepts only its own.
- **SC-4.3** (S, C) A holder's identity is portable without federation: the same principal and key log in at any server through the SSH signature grant.
- **SC-4.4** (S) A server reading another server is an anonymous client of its reads. There is no separate server-to-server protocol.
- **SC-4.5** (S) Any server MAY mirror version files, exports and signed acts of any namespace. A recipient checks each against its hash or signature.
- **SC-4.6** (C) Fetching is outside verification. What is fetched is committed to the repository, and a verifier reads only what is committed.
- **SC-4.7** (S) A server offers the version status read of Section 4, so that a revocation of a foundation reaches every store whose versions rest on it as a basis.

This is a network of projections. Each namespace has one canonical repository and one set of holders, so servers have nothing to reach consensus on, and none replicates another's state.

A moved foundation reaches every dependent with the same cause, not at the same instant. Each store sees it at its next fetch.

## 7. Conformance

There are two conformance classes, and each builds on a class of [LEDGER]. Conformance is behavioural: what is accepted, refused and committed.

| Class | Builds on | Conforms when it |
| --- | --- | --- |
| Server | The verifier class of [LEDGER] | Issues tokens as in Section 3, serves the reads of Section 4, refuses every invalid, relayed or stale envelope, and commits valid ones unchanged. |
| Client | The writer class of [LEDGER] | Completes each interactive grant, signs the expected bytes, and refuses a drifted batch. |

The test suite is part of this specification and is governed by the rules of [LEDGER]: RDF manifests, approval by the principal before a test case binds, an approved test case governing over the prose, and results published as implementation reports.

A test case here is an envelope, a token subject and a branch state, with the expected result: accepted, or the condition of Section 5.4 that refuses it.

Version 1.0 requires two independent implementations of each class that pass the approved test cases.

## 8. Security Considerations

The central property is that nothing a server or a token can do produces a ledger act. Each subsection below names a threat and how far that property holds against it.

### 8.1. Bearer Tokens

A stolen token can be replayed until it expires. It yields the reads its subject may make and the ability to submit envelopes, which still need that principal's signature. Tokens are short-lived and bound to one server's host.

### 8.2. Identity Providers

A compromised identity provider can assert any `email` claim and so obtain any principal's read access. It cannot forge an envelope's signature. A deployment that treats inbox contents as confidential should prefer the SSH signature grant.

### 8.3. The SSH Signature Grant

A captured login signature is useless after its nonce is spent or expired. Because the signing namespace differs from the acceptance namespace of [LEDGER], a login signature cannot be presented as an acceptance, and an acceptance signature cannot be presented as a login.

A key bound in any indexed namespace can log in. This grants no reads by itself, since roles are read from policy at each request.

### 8.4. Relayed and Swapped Envelopes

A holder who obtains another holder's signed envelope cannot submit it: the token's subject must equal the envelope's principal. A version that changes between review and signing is caught because the envelope names the version hash and the server re-reads the branch.

### 8.5. Server Compromise

A compromised server cannot forge an acceptance, review or revocation. It can withhold or delay reads and writes, and it can misuse its repository credential. Anything it commits that is not a valid signed entity fails verification under [LEDGER], but the credential may reach beyond ledger files. A deployment should scope that credential to pull-request branches.

### 8.6. Server-Held Signing Keys

Some deployments hold a holder's signing key in a service and release it on the holder's authenticated session. There a token does lead to a signature. The protocol is unchanged, because the server still receives and verifies an envelope, but the deliberate-act property then rests on the identity provider's step-up, not on possession of a key.

### 8.7. Stale Mirrors and Equivocation

A mirror cannot forge data, but it can serve an old export and withhold a revocation, so a review trigger never fires. A holder could also show different histories to different servers. Both are open (Appendix A).

## 9. Privacy Considerations

A server concentrates information that is spread across repositories.

- **Inbox reads reveal activity.** They show who proposed what, where and when. They are guarded by tokens.
- **Anonymous reads name holders.** An exported decision is served with the acceptances that carry holders' addresses.
- **Audit events are personal data.** They name principals and are held outside the repositories, under the server operator's control.
- **Identity providers learn of logins.** The SSH signature grant involves no third party.

## 10. IANA Considerations

This document has no IANA actions.

If it is ever submitted, it would request a grant type URI for the SSH signature grant, an authorization server metadata parameter for the nonce endpoint, and registration of its problem types.

## 11. References

### 11.1. Normative References

| Key | Title |
| --- | --- |
| LEDGER | Decision Ledger Protocol 1.0, Editor's Draft |
| RFC2119 | Key words for use in RFCs to Indicate Requirement Levels |
| RFC8174 | Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words |
| RFC6749 | The OAuth 2.0 Authorization Framework |
| RFC7636 | Proof Key for Code Exchange by OAuth Public Clients |
| RFC8252 | OAuth 2.0 for Native Apps |
| RFC8414 | OAuth 2.0 Authorization Server Metadata |
| RFC8628 | OAuth 2.0 Device Authorization Grant |
| RFC9457 | Problem Details for HTTP APIs |
| RFC9700 | Best Current Practice for OAuth 2.0 Security |
| OIDC-CORE | OpenID Connect Core 1.0 |
| SSHSIG | The SSH signature format, `PROTOCOL.sshsig`, OpenSSH |

### 11.2. Informative References

| Key | Title | Why it is cited |
| --- | --- | --- |
| RFC7521 | Assertion Framework for OAuth 2.0 Client Authentication and Authorization Grants | The pattern the SSH signature grant follows |

## Appendix A. Open Issues

- [ ] The grant type URI of the SSH signature grant.
- [ ] The nonce endpoint, its metadata parameter, and the token request's parameter names.
- [ ] Scope names.
- [ ] Resource paths and representations for the reads of Section 4.
- [ ] The envelope's wire format and the batch file's format.
- [ ] Problem type identifiers for the conditions of Section 5.4.
- [ ] Freshness: how a store learns that a mirror is behind.
- [ ] Non-equivocation: how divergent histories of one namespace are detected.
- [ ] Which identity providers an open server documents as examples.

## Appendix B. Changes

| Date | Change |
| --- | --- |
| 5 October 2026 | Split from sections 10 to 12 of the Decision Ledger Protocol Editor's Draft and recast in RFC form. Requirement identifiers unchanged. |
| 5 October 2026 | Added the steps of the SSH signature grant, the error conditions, and the security, privacy and IANA considerations. |
| 6 October 2026 | SC-2.5 and step 5 of Section 3.3: the login signature is verified against the key bindings of the indexed namespaces with their validity windows, not against `allowed_signers` (ruling 26). "Ground" in the ledger protocol's sense renamed "basis" in Section 4 and SC-4.7 (ruling 23). No section number of [LEDGER] cited here moved. Requirement identifiers unchanged. |
| 7 October 2026 | SC-3.7: the batch is the batch selection file of [LEDGER] Section 10.1 (ruling 30). No line here describes a pin as naming a server, so ruling 42 changes nothing in this document. Requirement identifiers unchanged. |
