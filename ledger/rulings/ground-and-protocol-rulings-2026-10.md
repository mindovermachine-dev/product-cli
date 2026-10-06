# Ground, trusted sources and decision classes: rulings and PRD section

4 October 2026 · Emil Klein

## Rulings, 4 October 2026

Eleven points were ruled on 4 October and ten on 5 October. All fifteen points proposed in discussion were accepted by the principal on 5 October and are listed below as accepted. This page is a draft until it is merged under `docs/` in product-cli; no earlier ruling is reopened.

### Ruled

1. A decision version may state the ground it is made on: other decision versions and external ground.
2. External ground is hashed. Every external ground carries a digest of the bytes it refers to.
3. Trusted sources exist for external ground. Trust decides whether a ground must be checked or may be accepted from the source.
4. Adding a trusted source is a decision proper: a version in a set, accepted normally. It is not a separate act entity.
5. Accepting a trusted-source decision requires a specific claim. Working name: `trust-source`.
6. The gate learns the required claim from the ontology. Field presence defines the decision class, and namespace policy maps classes to requirements. Both are used.
7. One class per version. Leaf classes are disjoint, and a version carrying two class-defining field groups is refused.
8. The set of write-gating classes is closed and ships with the vocabulary. Graph-stage classes may be opened to namespaces by a later ADR.
9. A stale decision ground is a review trigger on the `L012` pattern with a policy deadline. Listing was direct dependents only; ruling 18 of 5 October supersedes that part.
10. A version may be accepted before its grounds. An accepted version resting on a never-accepted ground fails `verify` as an `A` class.
11. Withdrawn trust follows the `L012` pattern. Grounds pinned before the withdrawal are listed for review; grounds pinned after it fail.

### Ruled, 5 October 2026

- **12.** The protocol exists so that independent implementations can be written: one in C#, and anyone's own. The specification and its test vectors are authoritative, and `ledger-core` is one conforming implementation.
- **13.** There are two protocols. The ledger protocol covers decisions and their schemas, verification and cryptographic integrity. The server-client protocol is separate and depends on it one way.
- **14.** The server-client protocol is built on OAuth 2.0.
- **15.** The server is always the token issuer. External identity providers are identity sources only.
- **16.** Namespaces are local to a server. They are data domains with no identity across the network, and a foreign namespace is pinned, not globally named.
- **17.** Convergence must be visible. Where chains of grounds across servers reach the same decision, that shared foundation shows, and its invalidation reaches every dependent together.
- **18.** Supersedes ruling 9 in part. A moved foundation is listed on every version whose ground closure contains it, with its distance. Only direct dependents must affirm or revise by the deadline.
- **19.** The test cases are part of the specification, as the SPARQL specification has its test suite. They are versioned and released with it, and no implementation owns them.
- **20.** Where the prose and an approved test case disagree, the test case governs until the defect is corrected.
- **21.** A test case binds only once the principal has approved it. A proposed test case binds no one.

### Accepted 5 October (consequences of rulings 12 to 18)

- The ledger protocol also covers authority: roles, grants, key bindings, policy, classes and claims.
- A foreign namespace is pinned by a trusted-source decision with the `signed` method: server and namespace as prefix, plus key material.
- A foreign decision is grounded on by its original version hash and never re-filed. A restatement grounds on the original.
- External ground converges on the byte digest, so the pinner and time stay out of a ground's identity.
- Pinning a foreign version brings in its ancestors' version files, so the foundation can be enumerated offline.
- The server-client protocol offers an anonymous status read by version hash.
- OAuth is profiled against RFC 9700. SSH login is an extension grant, agents use client credentials, and no scope confers a ledger act.

### Accepted 5 October (proposed 4 October)

- Grounds sit on the version as a hashed set of strings, so the acceptance covers them through the version hash.
- A decision ground pins a version hash, not a decision id.
- External ground is its own entity with a ULID and a content hash; the version references the hash.
- Held bytes are always digest-checked. A referenced ground is allowed only under a live trusted source.
- A source declares one of three methods: `content-addressed`, `signed`, `plain`.
- Policy may add requirements to a class and never go below the vocabulary's floor.
- Two live sources with overlapping prefixes and different methods are a conflict class.
- `A006` maps decision class to claim, not decision class to role.

## Terms

Nine terms carry the section. Names in code form are working names until the format document fixes them.

| Term | Meaning |
| --- | --- |
| Ground | What a decision version rests on. Stated on the version and hashed with it. |
| Decision ground | Another decision version, pinned by its version hash. |
| External ground | Anything outside the ledger, pinned by a digest of its bytes plus a locator IRI. |
| Held | The bytes are in the repository. `verify` checks the digest. |
| Referenced | The bytes are not in the repository. The digest is the pinner's attestation. |
| Source | An IRI prefix that external-ground locators fall under. |
| Trusted source | A source declared by a trusted-source decision with an unrevoked acceptance. |
| Decision class | A class of versions defined by presence of the version's own fields. |
| Claim | A capability in the closed `ledger:may` vocabulary. |

Trust covers origin, not relevance. Whether a ground supports a decision stays the acceptor's judgment.

## Format additions

Every addition is hashed when present and omitted when absent, so `CANONICAL_FORM` stays `v1` and no existing digest moves. A fixture-digest test proves it, as for keys in #67.

| Addition | Where | Form |
| --- | --- | --- |
| `grounds` | Version | Set of strings through `put_set`. Each entry is a version hash or a ground hash. |
| `source_prefix` | Version | One string, an absolute IRI prefix. Its presence defines the trusted-source class. |
| `source_method` | Version | `content-addressed`, `signed` or `plain`. Required with `source_prefix`. |
| `source_keys` | Version | Set of strings: key material for `signed` sources. Exact form open. |
| Ground entity | Log entry | Id `ground:<ULID>`, hash under `ledger.ground.v1`, payload `{locator, digest}`. |
| Held bytes | `.decisions/ground/<sha256>` | The pinned bytes, named by their digest. |
| `trust-source` | Role `may` | One new value in the closed capability vocabulary. |
| Class requirements | Namespace policy | Table keyed by class notation. Adds claims or the `-sk` requirement per class. |

The ground entity is identified by ULID and carries a content hash, as grants do, so ruling 1 of 1 October holds. The acceptance payload of ruling 4 is unchanged: it covers grounds through the version hash, and the version hash covers external ground through the ground hash.

A ground is unsigned, like a version. Anyone may file one, including an agent, and it has no effect until a version that references it is accepted.

### What trust decides

| Ground | Source | Result |
| --- | --- | --- |
| Held | Any | `verify` checks the digest. |
| Referenced | Trusted | Allowed. The digest is attested, not proven. |
| Referenced | Not trusted | Refused at the graph stage. |

### What a source's method means

| Method | Example | What `verify` can do offline |
| --- | --- | --- |
| `content-addressed` | Git commit, ledger version hash | The locator is the digest. Trust is about the publisher only. |
| `signed` | A foreign namespace's export under pinned key material | Check the signature. Cross-namespace decision grounds use this. |
| `plain` | A URL, a standards body | Nothing. Trust is a statement about who pinned it. |

## Vocabulary sketch

The class is defined by field presence, the claim is stated on the class, and no reasoner is needed. The emitter writes each `rdf:type` from one file, a shape checks the types agree with field presence, and SPARQL does the join.

```turtle
ledger:groundedOn a owl:ObjectProperty ;
    rdfs:subPropertyOf prov:wasDerivedFrom ;
    rdfs:domain ledger:DecisionVersion .

ledger:Ground a owl:Class ;
    rdfs:subClassOf prov:Entity .
ledger:locator a owl:ObjectProperty ;
    rdfs:domain ledger:Ground .
ledger:digest a owl:DatatypeProperty ;
    rdfs:domain ledger:Ground ; rdfs:range xsd:string .

ledger:TrustedSourceDecision
    rdfs:subClassOf ledger:DecisionVersion ;
    owl:equivalentClass [ owl:intersectionOf (
        ledger:DecisionVersion
        [ a owl:Restriction ;
          owl:onProperty ledger:sourcePrefix ;
          owl:minCardinality 1 ] ) ] ;
    skos:notation "trusted-source" ;
    ledger:requiresClaim ledger:cap-trust-source .

ledger:OrdinaryDecision
    rdfs:subClassOf ledger:DecisionVersion ;
    owl:disjointWith ledger:TrustedSourceDecision ;
    skos:notation "ordinary" ;
    ledger:requiresClaim ledger:cap-accept-decision .

ledger:cap-trust-source a skos:Concept ;
    skos:inScheme ledger:CapabilityScheme ;
    skos:notation "trust-source" .
```

`ledger:OrdinaryDecision` is defined by absence, which OWL cannot state under the open world. The shape states it with `FILTER NOT EXISTS`. That is sound here because the graph is a closed read model built from files.

`ledger-core` keeps its own class-to-claim table for the write path, since `accept` runs before the graph stage. A test holds that table equal to the vocabulary.

## Gates

The section adds five file-gate checks, five graph-stage classes and two review triggers. Class ids are not assigned here: each takes the next free number when its PR lands, and the closed-count tests change in the same PR.

### File gate, single file

| Check | Fails when |
| --- | --- |
| Ground reference syntax | A `grounds` entry is not a well-formed version hash or ground hash. |
| Source fields | `source_method` is outside the closed set, `source_prefix` is not an absolute IRI, or `source_keys` is present without `signed`. |
| One class per version | A version carries two class-defining field groups. Trivially true while one specialised class exists. |
| Ground entity | The locator is not an IRI or the digest is malformed. |
| Policy floor | A policy entry sets a class's requirement below the vocabulary's floor. |

### Graph stage, cross-file

| Class | Fails when |
| --- | --- |
| Dangling ground | A ground hash resolves to no version or ground entity in the repository's committed files. |
| Untrusted referenced ground | A ground's bytes are not held and no live trusted source covers its locator. |
| Unaccepted ground | An accepted version rests on a ground that has never had an acceptance. Ruling 10. |
| Source conflict | Two live sources have overlapping prefixes and different methods. |
| `A006` | An acceptance's actor has no live grant that `may` every claim the version's class requires, including policy additions. |

A held ground whose bytes do not match its digest also fails `verify`. It compares a file with repository content, as `L009` does with git, so its class letter is set against the format document.

### Review triggers, on the `L012` pattern

| Trigger | Listed when |
| --- | --- |
| Stale decision ground | A ground's version is revoked or is no longer its decision's tip. Listed on every version whose ground closure contains it, with distance. Only direct dependents must act. Rulings 9 and 18. |
| Ground under withdrawn trust | A referenced ground was pinned before its source's trust was withdrawn. Ruling 11. |

In both, the holder affirms, revises or holds the bytes, and a namespace policy may set a deadline after which the dependent stops being citable.

## Acts and derived state

No new act and no new signed entity is added. Grounds and sources ride on the existing acts, and every status below is computed at verify time and never stored.

| Act | What changes |
| --- | --- |
| File, revise | A version may state grounds and may carry source fields. Ground entities are filed alongside. Anyone may do this, including an agent. |
| Accept | The role check takes the required claims from the version's class plus policy additions. The signed payload is unchanged. |
| Revoke | Revoking the acceptance of a trusted-source version withdraws the trust. `expires_at` bounds it in time. |
| Verify | Computes ground status, source liveness and the classes above from the repository alone. |
| Inbox | Shows each ground with its trust status, and lists source decisions before the decisions grounded on them. |

A source is trusted while the tip version carrying its fields has an unrevoked, unexpired acceptance by a holder of `trust-source`. Widening a prefix is a new version and needs re-acceptance.

Because leaf classes are disjoint, `trust-source` stands alone for these versions. A holder of `accept-decision` only is refused.

### Agent path

An agent may propose a source and a decision grounded on it in one PR. The ground fails as untrusted until a holder accepts the source or the bytes are held, so the PR stays red for the right reason and the agent does not wait.

### Tests constraint 4 requires

- An agent identity cannot accept a trusted-source version.
- A non-interactive session cannot accept one.
- A holder with `accept-decision` and without `trust-source` cannot accept one.
- A policy entry cannot lower the claim a class requires.

## Migration and sequencing

This should land after Session B and before the Varve import (#73). Adding grounds or source fields to an existing decision is a new version and needs re-acceptance, the same migration note as for keys.

- **Varve import.** The 513 decisions in 79 interim set files are accepted at import. With grounds in the format first, they take their grounds at import at no extra cost. After it, each grounding is a re-acceptance. The interim files' `source-draft` field is a candidate ground.
- **Session B.** Signed acceptances (#70), `accept --batch` and the inbox (#79) are unaffected: the acceptance payload does not change. The role check gains the class argument, which touches Session A's function, so this is its own session.
- **First profile standard.** A profile's decisions rest on external grounds from a standards body, so that open item depends on this section.

Proposed order, each step one issue citing this section:

1. Vocabulary and shapes: `ledger:groundedOn`, `ledger:Ground`, the two leaf classes, `ledger:requiresClaim`, `trust-source`.
2. Version fields and their file-gate checks, with the fixture-digest proof.
3. Ground entity, held bytes and the digest check.
4. Class-to-claim role check, the policy table with its floor, and `A006`.
5. Graph-stage classes and the two review triggers.
6. Inbox display of grounds and trust status.

The format document and its migration note change in the same PR as each field, and exports are regenerated so `verify --export` passes.

## Upstream and open items

Three points go to the analyzers' repository as issues for the principal to file, and eight items are open or unconfirmed here.

### Upstream, `hafeok/decision-driven-analyzers`

- The export gains `ledger:groundedOn`, `ledger:Ground` nodes, the source predicates and two class types. The reader must tolerate them; it already keeps every `rdf:type` (#82).
- No citability change is asked of the reader. Ruling 10 puts the unaccepted-ground condition in `verify`, not in the generator.
- Trusted-source decisions have keys, so the generator would emit a type per source. Whether that is wanted is a question for that repository.

### Open or unconfirmed

- [ ] Confirmed 5 October, the reading of ruling 2: a digest of the bytes on every external ground, and the ground as an entity with its own content hash.
- [ ] Done 5 October: all proposed points accepted.
- [ ] Accepted 5 October: a ground's identity excludes the pinner and time. Whether the locator belongs in it is open, since mirrors of the same bytes should converge.
- [ ] The form of `source_keys`, and what is pinned for a foreign namespace: its policy hash, its `allowed_signers` snapshot, or its genesis grant.
- [ ] Whether the acceptance scope `class:<ref>` in the format document is the same notion as decision class. The format document was not read for this draft.
- [ ] The class ids: the next free numbers after what Sessions A and B landed.
- [ ] Where this lives in product-cli: new items in `docs/ledger-cli-prd.md` §0 plus a section, or its own rulings file.
- [ ] The wider protocol document: entity schemas for every entity, the act table and the wire artefacts. This section is one input to it.
