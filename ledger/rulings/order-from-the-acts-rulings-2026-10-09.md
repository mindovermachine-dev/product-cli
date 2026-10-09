# Rulings of 9 October 2026: order from the acts

Ruled by the principal on 9 October 2026. The numbers continue `namespace-design-rulings-2026-10-07.md`. They answer the twenty questions in section 7 of `ledger/prd/namespace-independence-prd.md` (the third revision, PR #142), and one question the review raised (ruling 101). They supersede D6 of 2 October and rulings 64, 65, 79, 83 and 84. They amend rulings 66 and 77, D7's first filer, and LP-4.39's "a window closes once". Ruling 82 stands.

**85.** Order comes from the acts. An act is before a terminating entry (a key's close, a grant's revocation or supersession, a policy) when the entry names it and the act's `at` is strictly earlier. An act the entry does not name is not before it, whatever its `at`. An enabling entry covers an act when its signed `at` is no later than the act's. Landing order decides nothing. This supersedes D6 of 2 October and ruling 64. (Q1, Q11)

**86.** A terminating entry names acts as `<id>@sha256:<hash>`, in a set-valued field `after` inside its signed payload. Policies do the same. (Q2, Q3, Q4)

**87.** Format 8 holds `after`, `anchor` and `role_hash`. There is no move act. An entry filed below format 8 names nothing. This amends ruling 66. (Q5, Q6)

**88.** A policy change names only the acts judged under the policy it replaces, since that policy. The earliest policy that names an act governs it. An act no policy names is under the policy in force at the tip. (Q7)

**89.** A namespace's genesis grant names its anchor: the hash of the genesis holder's self-bound binding. "First to land" plays no part. `init` refuses without a usable key, and `--without-key` is removed. A second genesis grant is `A005`, cleared by revoking the impostor's grant. Whether a pin keeps two key tokens (ruling 70) or one is left to the pinning design. This amends D7's first filer. (Q8)

**90.** Grants, grant acceptances, unavailabilities and availabilities are signed, with `at` in their payloads, before order from the acts is implemented (#82). `A006` judges a grant's grantor as of the grant. (Q9)

**91.** A grant names its role's content as `role_hash`, a digest under the prefix `ledger.role.v1`. A role's position plays no part. (Q10)

**92.** A name outside the entry's family, and a name that resolves to no filed act, are ignored and reported as notices. (Q12)

**93.** Removing every file of a namespace and its export in one commit is a notice, not `L007`. Removing part of a namespace stays `L007`. This supersedes ruling 65. (Q13)

**94.** `L009` judges only acts that no signature covers. This supersedes ruling 79. (Q14)

**95.** A move is a copy of a namespace's files. There is no move act, no landing record and no freeze. A namespace with no policy moves with its history carried, or is put under policy and re-accepted first. This supersedes rulings 83 and 84. (Q15)

**96.** `A007` returns to unused. This amends ruling 77. (Q16)

**97.** Ruling 82 stands as written. The legacy capability shrinks to `L007`, `L009` and the base overlay. (Q17)

**98.** The genesis holder's revoke takes an `at`, the time of compromise. It is bounded below by the `at` of the binding it closes. (Q18)

**99.** A writer that files a terminating entry names the acts of its family that the checkout holds, committed or not. A name that never lands is ignored under ruling 92. (Q19)

**100.** The case of two trusted self-bound bindings is closed by the anchor (ruling 89). It gets no separate fix under D6. (Q20)

**101.** The genesis holder's revoke may close a key that a `rotate` has already closed. Where several closes end one key, an act stands only if it is before each of them, as LP-4.39 says. So a thief's `rotate` that names forged acts does not keep them standing once the genesis holder's revoke, dated at the compromise, does not name them. This amends "a window closes once" for this case.
