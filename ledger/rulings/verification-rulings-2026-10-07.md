# Rulings of 7 October 2026: the verification findings

Ruled by the principal on 7 October 2026. The numbers continue `absorption-replies-rulings-2026-10-07.md`. Rulings 49 to 60 answer, in order, the twelve questions in section 14 of `ledger/sessions/2026-10-verification.md`.

**49.** A duplicate acceptance id, and a decision identity object filed more than once, are schema faults at verification.

**50.** A policy whose accept role is the genesis role is a schema fault at verification. So is a genesis role that carries a decision capability. A writer that opens a namespace refuses an existing root role that carries a decision capability. This extends D9 (f).

**51.** LP-9.14 and LP-9.15 are amended to what one namespace's export supports: a key's end date comes from the closes present in that export, and the limit this leaves is stated. An export-only verifier is built after the namespace-independence design, not before.

**52.** A sidecar on an act that no policy governs is a schema fault.

**53.** A `rotate` verifies only against the key it closes, and a writer signs a `rotate` with that key.

**54.** A forked decision has no latest version. The classes that judge the latest version skip it, and `G004` is its one finding.

**55.** A plain scalar that resolves to a float in a hashed string field is a schema fault. The correction is noted in Appendix C.

**56.** An explicit null in a required string field is absent, and so a schema fault. It is noted with ruling 55.

**57.** `L006` judges an escape's acceptor on every version, not only the latest.

**58.** A landed `format:` declaration is compared across history. Any change other than a raise to the lowest format the content needs, with nothing else changed, fails `L007`. This enforces the ruling of 5 October on #81.

**59.** A `set:` grant scope accepts every valid set id, dots included.

**60.** A change-set's `parents` is part of its header entity and is immutable once landed.
