# Rulings of 7 October 2026: the import triage and the namespace design

Ruled by the principal on 7 October 2026. The numbers continue `verification-rulings-2026-10-07.md`. Ruling 61 amends the rule on allocation at import. Rulings 62 to 80 answer, in order, questions Q1 to Q19 of `ledger/prd/namespace-independence-prd.md`. Ruling 81 follows from ruling 64.

## The import

**61.** Allocation on import comes from a triage the holder has reviewed. A cited decision takes its allocation from how it is cited. An uncited decision takes the allocation the triage proposes, where that proposal carries checked evidence. A decision the triage cannot place is imported unallocated, and `L001` fails for it. This amends the rule of 2 October that allocation is derived only from how a decision is cited.

## The namespace design

**62.** Each namespace has its own directory, `.decisions/ns/<namespace>/`.

**63.** Every store uses that layout. The flat layout is not a valid form of a store with one namespace.

**64.** Order before a move rests on a landing record that is written once, at departure, and fixed by a signed move act. The record is not held continuously. Before a move, order is read from the holding repository's history, as D6 says. This extends D6.

**65.** After a landed move act, the whole namespace may be removed from the source in one commit. Removing part of a namespace stays a finding. Adding the move act is within "no hash changes".

**66.** The layout is specification revision v1.9, with no format number. The move act is format 8. Pins take the pinning format, after it.

**67.** A genesis grant keeps the scope `*`, read as "this namespace".

**68.** There is no migration path for authority shared between namespaces. A governed store with more than one namespace, made before this lands, is re-founded per namespace.

**69.** A writer that closes a key in every namespace it holds files one change-set per namespace, in one commit. A key closed in one namespace and open in another is reported as a notice.

**70.** A pin's key material is the hash of the pinned namespace's genesis grant and the hash of its first trusted key binding.

**71.** Two unrelated namespaces of one name are told apart by genesis grant hash. A namespace holds at most one live pin per name.

**72.** Pinned material is a snapshot of the pinned namespace's export, held by the dependent. This holds inside one repository too.

**73.** An ungoverned namespace cannot be pinned.

**74.** Set and role IRIs carry the namespace: `urn:ledger-set:<ns>/<id>` and `urn:ledger-role:<ns>/<id>`.

**75.** A `supersedes` into another namespace, in a store that exists today, is judged on live claims only.

**76.** An unpinned `dec:` token of another namespace is refused from the pinning format on.

**77.** The new classes are `G007` for a dependency cycle, `G008` for a pin of a version that is not exported, and `A007` for a move mismatch. `A001`, `A002` and `A004` stay unused.

**78.** The pinned version itself carries `exported`.

**79.** `L009` for an arrived acceptance compares its actor with the author in the landing record.

**80.** A namespace with no dependency in either direction is reported as "isolated".

**81.** The export carries no landing ordinals and no introducing authors. Facts read from git stay out of the export, as ruled on 1 October.

## Questions the rulings raised

**82.** An implementation may read the flat layout in history, as a legacy capability. It is not part of the verifier profile. A verifier without it refuses a repository whose history predates v1.9; it never passes one by collapsing landing order. (N-Q1)

**83.** A namespace is frozen in its source from its move act on. An entity of it that lands there afterwards is `A007`, and a move act that lands after another entity of its namespace is refiled. (N-Q3)

**84.** A move act always exists and is signed where the namespace's policy requires a signature. In a namespace with no policy it is unsigned and unchecked, like every act there. (N-Q4)
