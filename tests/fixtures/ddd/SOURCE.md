# Vendored from decision-driven-design

These files are copied byte for byte, for the Varve admission test
(`tests/ProductCli.Canonical.Tests/VarveAdmissionTests.cs`, ADR-0004 D2, #167).
They are never edited here. `.gitattributes` marks them `-text`, so no checkout
changes their bytes. The test checks each one's sha256 against this file.

- **Repository:** `mindovermachine-dev/decision-driven-design`
- **Commit:** `799cbe84f2a010fb5b65a687eb253666948c73c8` (its `main` on
  2026-11-03, which carries D1.25). Both files were last changed in `0c94c32`.

| File here | Path there | sha256 |
|---|---|---|
| `acme-shop.ttl` | `applications/sdlc/what/examples/acme-shop.ttl` | `4018cb9ad45c13fceafd7e585649777e7c8b9e7231fab46a42eb3286df91b777` |
| `vectors.json` | `applications/sdlc/what/vectors.json` | `953666f42051e50632290f73200298c9b5d518d2a153c20a31f9589f94374a1d` |

`vectors.json`'s `graphHash`, the RDFC-1.0 hash of `acme-shop.ttl`, is
`sha256:10cd90893f7fadb1bdf80064a43795be95e2175684c6d3a93a7736abc7c2156b`.
