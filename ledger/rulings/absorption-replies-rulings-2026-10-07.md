Rulings of 7 October 2026: replies to the absorption session
Ruled by the principal on 7 October 2026. The numbers continue `basis-and-absorption-rulings-2026-10-06.md`. Rulings 27 to 40 answer, in order, the fourteen questions in §10 of `ledger/sessions/2026-10-protocol-absorption.md`. Rulings 41 to 48 follow from ruling 32.
27. The edge for a pinned basis is `ledger:pinnedBasis`. `ledger:basis` keeps its existing meaning on unavailability nodes.
28. The unavailability's `basis` field keeps its name.
29. Basis-loss is a report until a policy deadline passes. Past the deadline it is a failing class, added by the `L010` mechanism when implemented. Where policy sets no deadline it stays a report. This is ruling 9, and it supersedes the earlier statement that basis-loss is never a gate class.
30. SC-3.7 names the batch file of the ledger protocol as the batch.
31. The verifier profile requires the graph stage and the export check. A verifier that runs the file gate alone does not conform.
32. A namespace verifies the same wherever it sits. Sharing a repository with another namespace changes no rule, so a namespace can be moved to another repository by moving its files: no hash changes, no reference changes form, and both sides still verify. Coupling between namespaces is kept as low as it can be made.
33. The canonicalisation text follows the code: the whitespace stripped is space, tab, line feed, form feed and carriage return. Vertical tab is not stripped. No digest moves.
34. The pinned token forms are `dec:<ns>/<ULID>@sha256:<version hash>` and `basis:<ULID>@sha256:<basis hash>`. A version that carries one as a pinned basis declares a new format once pinning is implemented. Below that format the token is an opaque basis pointer.
35. Tokens of other schemes that carry `@sha256:` take no part in convergence for now.
36. There is one section map, in the protocol's appendix. The README names that appendix and holds no copy.
37. The protocol may cite the PRDs and the way-of-working document as rationale only. Anything an implementation needs is stated in the protocol.
38. The opening sentence of `basis-and-absorption-rulings-2026-10-06.md` that begins "Write this section, unchanged" is removed. It was an instruction to a session, not part of the rulings.
39. `ledger/prd/ledger-cli-prd.md` §0 item 4 and the two headings that cite the format document are kept as records.
40. The thirteen places where the code differs from its text go to a separate verification session. No code changes before then.
Namespace independence
41. A reference into another namespace has one form: a pinned token under a declared dependency. This holds inside a repository as it does across servers. It supersedes the statement that a repository may freely reference decisions in namespaces it does not own.
42. A dependency names what is trusted, not where it lives. The pin carries the other namespace's name and the key material that verifies its acts, and no server or location. This supersedes "server and namespace as prefix", accepted on 5 October.
43. Only `based_on` crosses a namespace. `supersedes` never does.
44. Only a decision marked `exported` can be pinned from another namespace.
45. No file holds entities of two namespaces.
46. Dependencies between namespaces are acyclic, and a cycle is a failing class. Instability, the outgoing dependencies over the incoming plus the outgoing, is reported for the namespaces of one repository. It is not a gate.
47. Authority is per namespace. Each namespace has its own genesis grant, roles, grants, key bindings and policy, and nothing in one namespace's authority has effect in another. This supersedes the ruling of 6 October that closing a key closes it in every namespace of the store, and the rule that the genesis holder's self-bound binding happens once per store. A writer may file a close in every namespace it holds.
48. Authority as its own unit, which namespaces depend on by pin, is intended for later. It is not designed now.
