# The ledger protocol test suite

**The suite is empty.** No test case has been written or approved yet.

A test case is an input and its expected result. The input is a store, a single
file or, for the server-client protocol, an envelope with a token subject and a
branch state. The expected result is a set of findings (compared as class and
subject pairs, never as text), a sequence of bytes, or a digest.

Manifests are RDF in the W3C test manifest vocabulary, as the SPARQL and SHACL
test suites use it. Each entry states its type, its action (the input), its
result (the expected output), the requirement ids it exercises, and its approval
status.

A test case binds only once the principal approves it. A proposed test case
binds no one.

An approved test case governs over the prose. Where the two disagree, the
specification has a defect, and until the prose is corrected the approved test
case decides.

The suite is part of the specification. It is versioned and released with it,
and no implementation owns it.
