# Rule-of-5 Review — kaya-8f4 "publication block vocabulary"

**Work Reviewed:** working-tree diff of `src/publication/mod.rs` (the publication block vocabulary, its validator, and the two plan-path call sites), `tests/publications.rs` (three new cases), `openspec/changes/add-publication-block-vocabulary/**` (proposal, tasks, the kaya-publications spec delta, the updated layer sidecar), the deployed `openspec/specs/kaya-publications/{spec.md,layer.md}`, and `docs/src/document-taxonomy.md`.

**Method:** `skill://rule-of-5-universal` lenses — correctness vs. the spec scenarios, invalid states made unrepresentable, error handling, boundaries, allocation/IO cost, test quality, and doc-truth. The defect was reproduced live during orientation (a page carrying `publication.produkt.reference` built and `check` reported no drift), and every fix was re-verified against the real binary.

**VERDICT: READY — CRITICAL count = 0 (HIGH 0, MEDIUM 0, LOW 2, both recorded).**

---

## STAGE 1: DRAFT (shape)

The decision the ticket asks for — own vocabulary and validator, versus a documented exemption — was settled by evidence, not preference. The orientation reproduced the defect: a page carrying a misspelled publication block survived `kaya publish` beside the fresh block, and `kaya check` reported no drift, because `plan` replaces only the block it names and the comparison is byte-wise. A documented exemption would bless exactly that state.

The chosen shape mirrors the research side: the publication layer validates its own pages against its own vocabulary, at plan time, before anything is written. The vocabulary is not a new list — it is derived from the tier vocabularies the layer already owns, so the scout's "third copy" risk does not arise.

Shape quality: GOOD.

## STAGE 2: CORRECTNESS

- The refusal carries the publication layer's own label (`kaya.publications.block_vocabulary`), names the offending block and the page, and leaves the page byte-identical — the same fail-closed shape as the research side's `kaya.documents.legacy_block_refused`.
- `plan_removal` validates before the strip, so a stale page for an omitted product cannot carry an unnamed block out of the build either.
- `check` inherits the refusal through `compare_publication` → `plan`, so the verifier fails closed exactly where the research side already does.
- Considered and accepted: `<flakyFailure>`/`<rerunFailure>` are attempt history, not outcomes; `<skipped>` maps to ignored; and a name whose tier does not agree with the tier the name resolves to is refused (tier-consistency), so `publication.package.overview` cannot masquerade as a product page.
- One boundary recorded, not silently kept: a publication page file whose name the build neither declares nor omits is never read, so its blocks are never validated. That is the build's pre-existing contract (it touches the pages it owns), unchanged by this ticket. Recorded as LOW-002.

Correctness quality: GOOD.

## STAGE 3: CLARITY

The refusal message states the rule (one publication.<tier>.<name> block per page, tier and name from the closed vocabulary) and names the offending block. The `outcome`-style docstrings carry the reasoning. The taxonomy doc now names both vocabularies and the per-layer refusal, so a reader of the corpus finds the boundary without archaeology.

Clarity quality: GOOD.

## STAGE 4: EDGE CASES

- A page for a collection-tier name: the vocabulary admits it (the collection tier is closed and stated in the spec), even though collections are unimplemented — forward-compatible with the collection work, consistent with the closed-vocabulary statement. Recorded as LOW-001.
- A page with no managed blocks: `previous.as_deref()` is None or the document carries no generated block; nothing to validate, nothing refused. ✓
- A block name with extra segments (`publication.product.overview.extra`): the tier/name split leaves a remainder that resolves nowhere → refused. ✓
- A page whose name is neither declared nor omitted: outside the build's contract; recorded, not changed (LOW-002).

Edge coverage: GOOD.

## STAGE 5: EXCELLENCE

- Allocation/IO: one extra scan per page at plan time, before any write; negligible, and the write path is unchanged.
- Test quality: the three new cases assert the labeled refusal, the offending block, the page, and byte-identical preservation — the same assertions the research-side vocabulary tests make.
- Consistency: the documents capability's requirement already scopes "a projected document" to its eleven kinds (its Family vocabulary requirement defines the term), so the deployed corpus now states each capability's own block vocabulary without contradiction; `document-taxonomy.md` carries the boundary for human readers.

Excellence: GOOD. Production ready: YES.

---

## Convergence

```
New CRITICAL issues: 0
Total new issues: 2 (both LOW, both recorded)
New issues vs Previous Stage: 0%
Estimated false positive rate: 0%
Status: CONVERGED
```

## Verdict

**READY** — CRITICAL 0, HIGH 0, MEDIUM 0, LOW 2 (both recorded boundaries, neither a defect of this change).
