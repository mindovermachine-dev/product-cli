# The C# native host — plan

Status: **accepted for H0**, 2026-09-29 — H0 runs first in the engagement week 1. Realises the re-decided
`dec/ddd/lsp-as-seam` (each language through its native semantic graph,
behind an LSP-shaped host). Nothing here is built.

## What changes

Today `ddd serve` reaches C# through the off-the-shelf
`roslyn-language-server`, and `ddd-lsp/src/adapter/csharp_facts.rs`
reconstructs visibility, attributes and signatures by slicing declaration
text around LSP symbol ranges. Writes are file-level text (`ddd_apply_edit`).

After: a small .NET process, **`ddd-roslyn`**, holds the solution in an
in-process Roslyn workspace (`MSBuildWorkspace`) and speaks LSP over stdio to
the unchanged Rust core. It answers the standard methods the language tools
already use, plus three `ddd/*` extensions for what the protocol cannot carry.

```
agent ──MCP──▶ ddd serve (Rust core: policy tables, interceptor, .ddd/, why)
                   │  LSP + ddd/* over stdio
                   ▼
               ddd-roslyn (.NET: MSBuildWorkspace, semantic model, DocumentEditor)
```

**Split of duties — the one rule that keeps `interceptor-not-extension`:**
the host reports facts and computes edits; it never decides what is contract
surface and never writes a file. Every write still goes through
`intercept::apply_edit`'s path — old text → new text → classify → apply or
reject — so there is one writer and one classifier for every language.

## The protocol

Standard, served from Roslyn directly: `initialize`/`initialized`,
`didOpen`/`didChange` (overlays — `revdiff` depends on them for historical
text), `documentSymbol`, `workspace/symbol`, `definition`, `references`,
`hover`, `signatureHelp`, `rename`, `publishDiagnostics`, and the readiness
notification the adapter waits on.

Extensions:

| Method | In | Out |
|---|---|---|
| `ddd/facts` | uri + optional text | `SymbolFacts[]` exactly as `ddd-core/src/surface.rs` defines them: effective (container-capped) visibility from `DeclaredAccessibility`, attributes from `GetAttributes()`, signature from a pinned `SymbolDisplayFormat`, `exported` from the configured endpoint attributes |
| `ddd/placements` | container query (type or namespace) | candidate containers with their symbol ids, files and the member kinds each admits |
| `ddd/addMember` · `ddd/replaceMember` | container symbol id + member source | the edited document text (new file or insert), formatted by `Formatter` — **returned, not written** |

`ddd/facts` takes text so the interceptor can ask for before *and* after
without touching disk.

## Rust side

- `Adapter.facts` becomes a two-way source: a text function (Bicep, Rust,
  HTML/CSS — unchanged) or a host method. C# switches to the host method;
  `csharp_facts.rs` is retired once parity holds, not before.
- `default_command` for C# becomes `["ddd-roslyn", "--stdio"]`;
  `.ddd/config.yaml` can still name `roslyn-language-server` as a fallback
  host, in which case the text-slicing facts stay in force.
- `ddd-mcp` gains `ddd_placements`, `ddd_add_symbol`, `ddd_replace_symbol`
  (results `applied | rejected | refused`), each calling the host for the
  edited text and then the existing intercept path. No argument names a file
  path. This *is* R2 of the engagement plan, re-based on the native host
  for C#.

## Milestones

| | Work | Est. | Exit |
|---|---|---|---|
| **H0** | Probe: `MSBuildWorkspace` loads the fixture solution and a customer-shaped one; time, memory, and adoption of a newly created `.cs` with diagnostics | 1 d | Recorded under `docs/g-track/`. **Go/no-go**: if it fails, week 1 stays on `roslyn-language-server` and this plan waits |
| **H1** | `ddd-roslyn` skeleton: LSP framing (StreamJsonRpc), lifecycle, overlays, the standard methods above | 2 d | `ddd warmup --language csharp` and every `ddd_*` language tool answer through it |
| **H2** | `ddd/facts` + the Rust two-way facts source | 1.5 d | **Parity run**: the M8 hand-labelled classifier corpus through both paths; every difference adjudicated as a text-slicing bug or a policy-row change, never silently accepted |
| **H3** | `ddd/placements`, `ddd/addMember`, `ddd/replaceMember`; the three `ddd_*` MCP tools | 2 d | A member added by symbol lands, is classified, and a surface-forming add is rejected with its demand |
| **H4** | Packaging: `ddd-roslyn` as a `dotnet tool`; `ddd` into cargo-dist | 0.5 d | Fresh machine: two installs, one `.mcp.json` entry, no Rust toolchain |

≈ 7 days. Against the engagement plan it **replaces** L0-min for C#
(`notify_created` against `roslyn-language-server`) and R2-min
(`insert.rs` text insertion): ≈ 4 d there, so net ≈ +3 d — and it removes
flaky dependency 1 (new-file adoption) rather than testing around it, since
the host adds the document to its own workspace.

## Layout and tests

- `ddd-roslyn/` at the repo root, its own solution and
  `Directory.Build.props` like `eval-dotnet/`, no reference into the rest of
  the repo — so it can be shared as one package.
- Its xunit suite drives it over stdio against the committed C# fixtures;
  it runs in the existing .NET CI job beside `eval-dotnet` and `spec-flow`.
- Workspace Rust CI stays hermetic per `dec/ddd/fixtures-not-sdk`: the mock
  host learns the three `ddd/*` methods, so the Rust half is tested without
  the SDK. The real-host run is `DDD_LSP_E2E=1`, as today.

## Settled 2026-09-29

1. The re-decided `dec/ddd/lsp-as-seam` wording is accepted; its ledger
   version is filed (`cs:01M3PHNSVZMME6XK72GNHHNKV6`) and awaits the
   principal's signature.
2. `DDD-arch-02` is narrowed to Bicep. This moves the pin
   `dec/ddd/rust-host-is-real` holds on it, which now reports as basis loss
   until that decision is revisited.
3. H0 runs first in week 1, ahead of L0-min; on go, H1–H3 replace L0-min and
   R2-min for C#.
