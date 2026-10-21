# Build-seam — worked message pair

Two example messages that validate against `workunit.schema.json` and
`verdictevent.schema.json` — the local projections of the normative
[ai-development-contracts](https://github.com/Hafeok/ai-development-contracts)
seam contracts (v0.1.0), which govern.

A WorkUnit is a **complete, executable SPMC package**. The producer — this
specification framework — authors the whole thing: the fully-pinned model
binding (**M**, unit-level, guaranteed by tier homogeneity), the
content-addressed context pool (**C**, inlined by value), and the sealed
`cell-graph` whose cells each carry their own output shape (**S**) and prompt
(**P**) inline. An executor holding the unit can run every cell with **no
further input** and resolves nothing by calling back — the freeze is the
decoupling.

## 1. Emitted WorkUnit (specification → execution)

```json
{
  "unit-ref": "wu-checkout-refund-decider-0007",
  "parent-deliverable": "deliverable-refunds",
  "bundle-hash": "sha256:b3d1f2a9c7e84416a0f5d2c8e1b6079a4d33f8e2c1a90b7765e4d3c2b1a09f8e7",
  "tier": "constrained-implementer",
  "acceptance-class": "needs-verdict",
  "ladder-position": 0,
  "artifact-delivery": {
    "mode": "workspace",
    "workspace": {
      "kind": "git",
      "location": "file:///workspaces/checkout",
      "ref": "main"
    }
  },
  "spmc-bundle": {
    "model": {
      "capability-tag": "constrained-implementer",
      "binding": {
        "provider": "example-endpoint",
        "model-id": "coder-m",
        "revision": "2026-06",
        "architecture": "dense",
        "quantization": "fp8",
        "invocation": { "temperature": 0, "top_p": 1, "max_tokens": 4096 }
      }
    },
    "context-pool": {
      "bundle-form-profile": "n-quads-canonical/urdna2015",
      "fragments": [
        {
          "id": "frag:order-decider",
          "role": "decision",
          "media-type": "text/markdown",
          "content": "Order#decide(order, cmd): pure function; no I/O, no persistence.",
          "provenance": { "graph-node": "Order#decide", "version": "3" }
        },
        {
          "id": "frag:invariant-refund-1",
          "role": "constraint",
          "content": "ORDER-REFUND-1: refund_total + amount <= paid_total"
        },
        {
          "id": "frag:simulation",
          "role": "simulation",
          "content": [
            { "given": ["paid 100", "refunded 60"], "when": "refund 50", "then": "Rejected(ORDER-REFUND-1)" },
            { "given": ["paid 100", "refunded 60"], "when": "refund 40", "then": "Accepted[ev-refund-issued]" }
          ]
        }
      ]
    }
  },
  "cell-graph": {
    "cells": [
      {
        "id": "c-test",
        "requires": [],
        "schema": {
          "shape-language": "JSON Schema",
          "shape-version": "2020-12",
          "document": { "type": "object", "required": ["tests"] }
        },
        "prompt": {
          "content": "Write failing unit tests for the Order refund Decider from the frozen simulation scenarios."
        },
        "context-refs": ["frag:order-decider", "frag:invariant-refund-1", "frag:simulation"],
        "output": {
          "artifact-id": "art:refund-decider-tests",
          "media-type": "text/x-typescript",
          "path": "src/order/refund.decider.test.ts"
        }
      },
      {
        "id": "c-impl",
        "requires": ["c-test"],
        "schema": {
          "shape-language": "JSON Schema",
          "shape-version": "2020-12",
          "document": { "type": "object", "required": ["code"] }
        },
        "prompt": {
          "content": "Implement a pure decide(order, cmd) so the provided tests pass. No I/O, no persistence. Reject ORDER-REFUND-1 when refund_total + amount > paid_total."
        },
        "context-refs": ["frag:order-decider", "frag:invariant-refund-1"],
        "output": {
          "artifact-id": "art:refund-decider-impl",
          "media-type": "text/x-typescript",
          "path": "src/order/refund.decider.ts"
        }
      }
    ]
  }
}
```

Every field above is part of the seam contract — there is no separate "opaque
executor slot." Tier, escalation-ladder position, artifact transport, the model
binding, and the sealed cell-graph are all **producer-authored** (producer-owns,
foundation RFC 0001): the specification pillar froze and handed over the
artifact, so it defines the artifact's shape. `bundle-hash` is the hash of
`spmc-bundle` taken over the declared `bundle-form-profile`, and it *is* the
unit's identity. The cell-graph is sealed — no `requires` edge names a cell
outside this unit — and every `context-refs` id resolves in the unit's
`context-pool`.

## 2. Returned VerdictEvent (execution → stream)

```json
{
  "event-id": "ev-9f81c0a4-7b22-4e6d-9b3a-1c2d3e4f5a6b",
  "emitted-at": "2026-06-26T02:14:08Z",
  "unit-ref": "wu-checkout-refund-decider-0007",
  "parent-deliverable": "deliverable-refunds",
  "bundle-hash": "sha256:b3d1f2a9c7e84416a0f5d2c8e1b6079a4d33f8e2c1a90b7765e4d3c2b1a09f8e7",
  "verdict": "accepted",
  "tier-ran": "constrained-implementer",
  "next-consequence": "advance",
  "cell-results": [
    {
      "cell-id": "c-test",
      "verdict": "accepted",
      "artifact": {
        "artifact-id": "art:refund-decider-tests",
        "content-hash": "sha256:11aa22bb33cc44dd55ee66ff77008899aabbccddeeff00112233445566778899",
        "media-type": "text/x-typescript",
        "delivery": { "mode": "workspace", "path": "src/order/refund.decider.test.ts", "ref": "a1b2c3d" }
      }
    },
    {
      "cell-id": "c-impl",
      "verdict": "accepted",
      "artifact": {
        "artifact-id": "art:refund-decider-impl",
        "content-hash": "sha256:99887766554433221100ffeeddccbbaa99887766554433221100ffeeddccbbaa",
        "media-type": "text/x-typescript",
        "delivery": { "mode": "workspace", "path": "src/order/refund.decider.ts", "ref": "a1b2c3d" }
      }
    }
  ]
}
```

The `emitted-at` of `02:14:08Z` is a night emission — it sat in the stream until
the consumer next read it, with no executor-side wait (temporal decoupling). The
`bundle-hash` matches the unit exactly, so the `accepted` verdict is
attributable to the immutable input it ran against. Because the WorkUnit
declared `artifact-delivery.mode = workspace`, each produced artifact returns as
a `path` + `ref` into the declared git clone, still carrying its own
`content-hash` — provenance holds regardless of transport. Had the unit declared
`mode = inline`, each artifact's `delivery` would instead carry `content` by
value and the seam would stay store-free.

## What an execution framework must do to be conformant (spec §5.1)

1. Given a WorkUnit, walk the `cell-graph` and reach a declared `verdict` with
   **no further input** — every prompt, shape document, and context fragment is
   already inline, so nothing is resolved by calling back.
2. Emit the verdict as a **self-describing** VerdictEvent — its `unit-ref`,
   `parent-deliverable`, and `bundle-hash` are sufficient for a consumer that
   never saw the emission to reconcile it.
3. Hold **no knowledge of any consumer** — emit to the stream and be done.
4. Return produced artifacts **per the WorkUnit's declared `artifact-delivery`
   mode**, each content-hash identified, so every accepted artifact is
   attributable to an immutable input.

Scheduling, admission, batching, and the transport of the event stream (log,
bus, or file) remain the executor's own concern — the contract fixes the two
message shapes and the freeze-before-emit obligation, and nothing about the
executor's internals.
