<!-- markdownlint-disable MD051 -->
<!-- MD051 (fragment link targets) disabled intentionally: fragment links use
     short-form #d-xxxx targets which do not match full heading slugs.
     Link integrity is verified by lychee. Remove when heading format is
     migrated to match short-form targets with decision automation. -->
# Decision Register

## What Is the Decision Register?

The decision register captures **scoped observations and their direct consequences** — findings that emerged from reading the spec, working through Rust language constraints, or resolving a specific domain modelling choice. Each entry is small enough that a full ADR would be disproportionate, and specific enough that it belongs to one area rather than the whole architecture.

Entries span SDMX specification observations, Rust language behaviour, and domain modelling choices.

## When to Add an Entry

Add a register entry when:
- A spec reading produces a concrete field-level or type-level consequence
- A Rust language constraint forces a specific implementation choice
- A domain modelling question is resolved and the resolution should be citable

Adding an entry is a three-step operation:
1. Add a row to the [Entry Index](#entry-index) — ID, Area, and one-line title
2. Add the full entry under [## Entries](#entries) using the template below
3. Update the `<!-- Next ID: -->` comment in the index footer to the following ID (e.g. D-0005 → D-0006)

## Relationship to ADRs and Design Documents

- **ADRs** record cross-cutting architectural commitments that constrain the whole system and are expensive to reverse. If a decision introduces a new cross-cutting constraint, write an ADR in `docs/adr/` instead.
- **Design documents** explore the design space before a decision is made. A register entry may cite a design doc as the source of the discussion that produced it.
- **Entries that cite an ADR as their source** signal that they are consequences of that architectural decision, not independent commitments.

See [ADRs](adr/README.md) and [Design Documentation](design/README.md).

## Entry Template

**Step 1** — add this row to the [Entry Index](#entry-index) (replace NNNN, Area, and Title), then update the `<!-- Next ID: -->` footer:

~~~markdown
| [D-NNNN](#d-nnnn) | Area | Title |

<!-- Next ID: D-NNNN+1 -->
~~~

**Step 2** — add the full entry under [`## Entries`](#entries). Copy the block below and remove inapplicable rows and sections. `Spec ref`, `Source`, `Related`, `Rationale`, and `Consequences` are omitted entirely when not applicable — do not leave them blank.

~~~markdown
### D-NNNN — Title <!-- Short declarative title: what was decided, not what was observed -->

| **Area**     | | <!-- Domain area: Annotation, Identifiers, Collections, Serde, etc. -->
| **Phase**    | | <!-- Milestone when decided: M0, Phase-1, Phase-2, etc. -->
| **Status**   | | <!-- Active · Superseded(D-NNNN) · Deprecated -->
| **Keywords** | | <!-- Comma-separated conceptual tags, no backticks: validation, serde, no_std -->
| **Spec ref** | | <!-- Link to the XSD or spec section that is the primary evidence -->
| **Source**   | | <!-- Link to the design doc or ADR where this was discussed, if any -->
| **Related**  | | <!-- [D-NNNN](#d-nnnn) links to closely related decisions, if any -->

**Observation**: What the spec, language, or runtime does that forces a choice.

**Decision**: The ruling — what the code does as a result. One to three sentences.

**Rationale**: Why this option over the alternatives. Omit if the decision is self-evident from the observation.

**Consequences**: Downstream implications, open questions, or deferred follow-ups. Omit if none.
~~~S

> **Amended entries retain their full body.** When a later entry changes an earlier one, the earlier entry keeps its body verbatim (the audit trail) and records the relationship in two places: a blockquote at the top of the body, and a decoration on its Entry Index title. Use one of four relationship verbs:
>
> | Verb           | Meaning                                                                                          | Status cell                          | Index title decoration                |
> |----------------|--------------------------------------------------------------------------------------------------|--------------------------------------|---------------------------------------|
> | **Superseded** | The whole decision is replaced; the body is now historical only.                                 | `Superseded(D-NNNN)`                 | `~~title~~ (superseded by D-NNNN)`    |
> | **Amended**    | The body stands; a later entry changes or withdraws one clause, or revises the mechanism.        | `Active (… clause amended by D-NNNN)`| `title (amended by D-NNNN)`           |
> | **Corrected**  | The body stands; a later entry fixes a factual error in it.                                      | `Active (… corrected by D-NNNN)`     | `title (corrected by D-NNNN)`         |
> | **Promoted**   | The decision is lifted into an ADR; the entry is kept as the audit trail.                        | `Promoted(ADR-NNNN)`                 | `title → promoted to ADR-NNNN`        |
>
> - **Superseded** strikes the title (`~~…~~`) because the decision is dead; the other three leave the title intact because the decision still stands.
> - The **body blockquote** carries the detail (dated, naming the superseding/amending entry and what changed); the Status cell and index decoration are the at-a-glance index. List multiple amenders comma-separated: `(amended by D-NNNN, D-MMMM)`.
> - An entry may carry more than one relationship (e.g. promoted to an ADR *and* a clause amended later); the Status cell leads with the dominant verb and notes the secondary in parentheses.
> - **Re-homing** a decision under another register entry (the mechanism moves but the finding stands) is recorded as an **Amended**, with the shape change explained in the body note — it is not a separate verb.

## Entry Index

| ID                | Area                        | Title                                                                                                                                                                                                                             |
|-------------------|-----------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| [D-0001](#d-0001) | Annotation                  | Annotation text is fully optional                                                                                                                                                                                                 |
| [D-0002](#d-0002) | Reference types             | Reference types kept distinct                                                                                                                                                                                                     |
| [D-0003](#d-0003) | Codelist                    | Hierarchical codes use flat mapping                                                                                                                                                                                               |
| [D-0004](#d-0004) | Identifiers                 | Identifier validation at construction → promoted to ADR-0021                                                                                                                                                                      |
| [D-0005](#d-0005) | Encapsulation               | Invariant-bearing types use private fields and custom Deserialize → promoted to ADR-0021                                                                                                                                          |
| [D-0006](#d-0006) | Collections                 | ~~BTreeMap used throughout~~ (superseded by [D-0051](#d-0051))                                                                                                                                                                    |
| [D-0007](#d-0007) | String ownership            | Owned String for all text fields → promoted to ADR-0022                                                                                                                                                                           |
| [D-0008](#d-0008) | DateTime typing             | chrono::DateTime for date-time fields                                                                                                                                                                                             |
| [D-0009](#d-0009) | Maintainable artefacts      | isFinal removed                                                                                                                                                                                                                   |
| [D-0010](#d-0010) | Maintainable artefacts      | isPartialLanguage added (amended by [D-0052](#d-0052))                                                                                                                                                                            |
| [D-0011](#d-0011) | Annotation                  | AnnotationURL is a vec of structs                                                                                                                                                                                                 |
| [D-0012](#d-0012) | Data structure              | AttributeRelationship is a structured enum                                                                                                                                                                                        |
| [D-0013](#d-0013) | Constraints                 | AvailabilityConstraint carries no MaintainableMetadata                                                                                                                                                                            |
| [D-0014](#d-0014) | Identifiable artefacts      | uri added to IdentifiableMetadata (corrected by [D-0035](#d-0035))                                                                                                                                                                |
| [D-0015](#d-0015) | Data structure              | MeasureList is optional (corrected by [D-0025](#d-0025))                                                                                                                                                                          |
| [D-0016](#d-0016) | Localisation                | LocalisedString rejects blank keys and empty values (amended by [D-0031](#d-0031), [D-0051](#d-0051), [D-0059](#d-0059))                                                                                                          |
| [D-0017](#d-0017) | Encapsulation               | Field visibility rule → promoted to ADR-0021 (amended by [D-0026](#d-0026))                                                                                                                                                       |
| [D-0018](#d-0018) | Conventions                 | bool vs enum chosen by call-site visibility                                                                                                                                                                                       |
| [D-0019](#d-0019) | Data structure              | AttributeRelationship data variants wrap validating newtypes (amended by [D-0077](#d-0077))                                                                                                                                       |
| [D-0020](#d-0020) | Identifiers                 | Identifiers validated at declaration, not at reference (amended by [D-0077](#d-0077))                                                                                                                                             |
| [D-0021](#d-0021) | Conventions                 | #[non_exhaustive] per public enum, not blanket                                                                                                                                                                                    |
| [D-0022](#d-0022) | Serialisation               | ~~Round-trip fidelity is semantic, not byte-level~~ (superseded by [D-0031](#d-0031), residual clause by [D-0052](#d-0052))                                                                                                       |
| [D-0023](#d-0023) | Identifiers                 | Identifier validation is per-artefact lexical type, not blanket NCName (amended by [D-0052](#d-0052), [D-0077](#d-0077))                                                                                                          |
| [D-0024](#d-0024) | Versionable artefacts       | version is optional (`Option<Version>`); un-versioned is distinct (amended by [D-0027](#d-0027))                                                                                                                                  |
| [D-0025](#d-0025) | Data structure              | DSD has multiple measures (3.x), not a single PrimaryMeasure (2.1) (amended by [D-0049](#d-0049), [D-0051](#d-0051), [D-0052](#d-0052), [D-0057](#d-0057))                                                                        |
| [D-0026](#d-0026) | Constraints                 | CubeRegion modelled to full spec structure (dim/component, cascade, time range) (corrected by [D-0038](#d-0038); amended by [D-0051](#d-0051), [D-0052](#d-0052), [D-0064](#d-0064))                                              |
| [D-0027](#d-0027) | Lexical types               | Validated lexical newtypes (SdmxDecimal/Integer/Version/TimePeriod); lossless raw + retained discriminant (amended by [D-0070](#d-0070), [D-0073](#d-0073))                                                                       |
| [D-0028](#d-0028) | Data structure              | Component Representation subsystem (Enumeration/TextFormat, DataType, facets) (amended by [D-0048](#d-0048), [D-0052](#d-0052))                                                                                                   |
| [D-0029](#d-0029) | Data structure              | TimeDimension modelled as a separate Option slot on the DSD                                                                                                                                                                       |
| [D-0030](#d-0030) | Maintainable artefacts      | External-reference modelled as an Infoset Store + derived view (amended by [D-0031](#d-0031), [D-0052](#d-0052))                                                                                                                  |
| [D-0031](#d-0031) | Architecture (foundational) | Two-layer model: Infoset Store + derived views; never collapse the store → promoted to ADR-0023 (amended by [D-0059](#d-0059))                                                                                                    |
| [D-0032](#d-0032) | Item schemes                | ItemScheme.isPartial modelled on ItemScheme (not MaintainableMetadata); distinct from isPartialLanguage (amended by [D-0052](#d-0052))                                                                                            |
| [D-0033](#d-0033) | Annotation                  | Annotations modelled on every AnnotableType descendant (universal extension point); via IdentifiableMetadata if identifiable, else bare field (amended by [D-0049](#d-0049))                                                      |
| [D-0034](#d-0034) | Constraints                 | ConstraintAttachment split into two per-constraint enums (amended by [D-0044](#d-0044))                                                                                                                                           |
| [D-0035](#d-0035) | Identifiable artefacts      | Link modelled on IdentifiableMetadata (reverses [D-0014](#d-0014)'s omission); typed multi-valued association, not transport-layer                                                                                                |
| [D-0036](#d-0036) | Constraints                 | ReportingConstraint cube regions capped at 2 (CubeRegions newtype) — mechanical maxOccurs; include/exclude pairing left to a lint                                                                                                 |
| [D-0037](#d-0037) | Constraints                 | DataConstraint carries the 3.0 role (Option of ConstraintRole) as a superset member; ReportingConstraint renamed DataConstraint                                                                                                   |
| [D-0038](#d-0038) | Constraints                 | Member selections modelled to full MemberSelectionType (CubeRegionKey/ComponentValueSet); non-empty Values enforced; corrects [D-0026](#d-0026) (amended by [D-0051](#d-0051), [D-0052](#d-0052); corrected by [D-0064](#d-0064)) |
| [D-0039](#d-0039) | Constraints                 | DataKeySet subtree modelled on DataConstraint; 3.1 multi-value keys carried as superset; fixed=true include attributes not stored (amended by [D-0051](#d-0051), [D-0052](#d-0052))                                               |
| [D-0040](#d-0040) | Constraints                 | CubeValue split into spec-exact CubeKeyValue/SimpleComponentValue carrying per-value cascade/lang/validity; CubeValues newtype split (amended by [D-0052](#d-0052))                                                               |
| [D-0041](#d-0041) | Constraints                 | DataConstraint.attachment is Option (ConstraintAttachment minOccurs=0, both versions); availability attachment stays mandatory                                                                                                    |
| [D-0042](#d-0042) | Constraints                 | ReleaseCalendar (3.0-only) carried on DataConstraint as a superset member; three required xs:string fields, unvalidated                                                                                                           |
| [D-0043](#d-0043) | Constraints                 | series_count/obs_count stored as Option of i32 (xs:int, verbatim); rule stated — integer types mirror the XSD value space                                                                                                         |
| [D-0044](#d-0044) | Constraints                 | 3.0-only data-source attachment members modelled (SimpleDataSource arm; QueryableDataSource companions); amends [D-0034](#d-0034)'s 3.1-only count                                                                                |
| [D-0045](#d-0045) | Data structure              | 3.1-only DimensionConstraint (Dataflow) and evolvingStructure (DSD) carried as superset members (amended by [D-0052](#d-0052), [D-0077](#d-0077))                                                                                 |
| [D-0046](#d-0046) | Architecture                | 3.0↔3.1 divergences resolved by carrying the superset; disposition table is the reconciliation baseline (amended by [D-0077](#d-0077))                                                                                            |
| [D-0047](#d-0047) | Codelist                    | ValueList modelled as a maintainable artefact (not an item scheme); fourth id tier (plain xs:string); items a Vec — duplicates are wire-valid                                                                                     |
| [D-0048](#d-0048) | Data structure              | Representation completed: EnumerationReference widened, pattern/isMultiLingual/occurs drawn, per-position rules constructor-enforced (amended by [D-0052](#d-0052))                                                               |
| [D-0049](#d-0049) | Data structure              | DSD container redrawn: identifiable descriptors (DimensionList/Group/AttributeList/MeasureList); DSD itself becomes a derived carrier (amended by [D-0051](#d-0051), [D-0052](#d-0052), [D-0077](#d-0077))                        |
| [D-0050](#d-0050) | Data structure              | MetadataAttributeUsage and MeasureRelationship modelled on the attribute list (amended by [D-0077](#d-0077))                                                                                                                      |
| [D-0051](#d-0051) | Collections                 | Wire collections stored as ordered Vecs (order + duplicates preserved); lookup is a first-match view; supersedes [D-0006](#d-0006)                                                                                                |
| [D-0052](#d-0052) | Architecture                | Attribute statedness stored: XSD defaults and fixed values are views, not data; Option + effective views; fixed mismatch rejected (amended by [D-0057](#d-0057))                                                                  |
| [D-0053](#d-0053) | Dataflow                    | Dataflow.dsd is Option by design: Structure is minOccurs=0 (external-reference stubs); the prose conditional is lint territory                                                                                                    |
| [D-0054](#d-0054) | Codelist                    | CodelistExtension modelled on Codelist (ref + prefix + inclusive/exclusive member selection); geo-codelist artefacts recorded out of scope                                                                                        |
| [D-0055](#d-0055) | Organisation                | Contact modelled on Agency (names/departments/roles + one interleaved detail list); other organisation kinds remain out of scope                                                                                                  |
| [D-0056](#d-0056) | Data structure              | effective_position pinned 1-based: the derived fallback is list index + 1, matching official stated-position samples; lint now writable                                                                                           |
| [D-0057](#d-0057) | Data structure              | Component id statedness stored (ComponentMetadata leaf); the trait id() is the effective view; TimeDimension fixed id enforced                                                                                                    |
| [D-0058](#d-0058) | Data structure              | AttributeRelationship dimension refs carry the per-ref optional attribute (DimensionRef); statedness stored; closes the superset hole (amended by [D-0077](#d-0077))                                                              |
| [D-0059](#d-0059) | Localisation                | LocalisedString key: statedness stored + blank/off-pattern keys held; parsable-within-spec reject-line (amended by [D-0066](#d-0066))                                                                                             |
| [D-0060](#d-0060) | Lexical types               | SdmxVersion ordering deferred past Phase 1: raw-based Eq only, no Ord/PartialOrd; SemVer precedence is a future method/wrapper, not an Ord impl (amended by [D-0070](#d-0070))                                                    |
| [D-0061](#d-0061) | Codelist                    | MemberValue content held verbatim (carrier); WildcardedMemberValueType well-formedness (non-empty + pattern) is a Layer-2 lint, not a new() check                                                                                 |
| [D-0062](#d-0062) | Item schemes                | ItemSchemeArtefact trait deferred to its first generic consumer (build-at-first-caller); wrappers forward is_partial/get/iter via inherent methods                                                                                |
| [D-0063](#d-0063) | Serialisation               | Derived serde is an internal lossless projection, not the SDMX wire format; wrappers serde(transparent); convergence deferred to a Phase-2 gate (amended by [D-0068](#d-0068))                                                    |
| [D-0064](#d-0064) | Constraints                 | TimeRange remodelled to { kind, valid_from, valid_to }; carries TimeRangeValueType's wrapper validFrom/validTo, the validity arm D-0038 missed                                                                                    |
| [D-0065](#d-0065) | Conventions                 | Hash/Eq/PartialEq derived uniformly wherever float-free; SdmxVersion hand-writes Hash over its raw string (Eq/Hash contract) (amended by [D-0070](#d-0070))                                                                       |
| [D-0066](#d-0066) | Localisation                | LocalisedString element is the named LocalisedText { language, text }, not an anonymous tuple; pub-field carrier; amends [D-0059](#d-0059)'s store shape                                                                          |
| [D-0067](#d-0067) | Item schemes                | ItemScheme kept a public, invariant-light generic carrier; the wrappers own the construction invariants, so exposure bypasses no validation                                                                                       |
| [D-0068](#d-0068) | Serialisation               | Internal serde projection never converges to the wire; round-trip verified through a non-wire format, serde_json dropped; resolves [D-0063](#d-0063)'s deferral                                                                   |
| [D-0069](#d-0069) | Architecture                | Reference, version, and time-period grammars are model surface gating the 0.1.0 publish; the wire mapping stays with the parser/writer                                                                                            |
| [D-0070](#d-0070) | Lexical types               | SdmxVersion raw-free: canonical grammar, statedness-preserving decomposition; amends [D-0027](#d-0027)/[D-0060](#d-0060)/[D-0065](#d-0065)                                                                                        |
| [D-0071](#d-0071) | Lexical types               | VersionRef models the version reference grammar (WildcardVersionType); one + wildcard enforced across editions                                                                                                                    |
| [D-0072](#d-0072) | Lexical types               | ObservationalTimePeriod union carries TimePeriodRange.period; SdmxTimeRange models the TimeRangeType lexeme                                                                                                                       |
| [D-0073](#d-0073) | Reference types             | Reference types own their class URN (Display/FromStr); versions typed VersionRef; + admitted everywhere, * nowhere                                                                                                                |
| [D-0074](#d-0074) | Lexical types               | `PartialEq<str>` string identity on the lexeme-storing types only; raw-free grammar types take none (canonical semantics pre-agreed)                                                                                              |
| [D-0075](#d-0075) | Conventions                 | Schema-unbounded integers take u32 width where the value is a count or version component; the bound is a recorded deviation; lexeme newtypes where the value is the datum                                                         |
| [D-0076](#d-0076) | Data structure              | Format-facet validity moves into the field types: time_interval takes the SdmxDuration newtype, the positiveInteger facets and MaxOccurs::Count take NonZeroU32; min_occurs stays u32                                             |
| [D-0077](#d-0077) | Identifiers                 | Local reference ids validate their lexical tier at construction (edition union where divergent); D-0020 narrows to referential integrity; DimensionRef/MetadataAttributeUsage/Code promoted                                       |

<!-- Next ID: D-0078 -->

## Entries

### D-0001 — Annotation text is fully optional

| **Area**     | Annotation |
| **Phase**    | M0 |
| **Status**   | Active |
| **Keywords** | annotation, optionality, spec-alignment |
| **Spec ref** | [SDMXCommon.xsd 3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXCommon.xsd#L215-L251) + [3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXCommon.xsd#L219-L255) |

**Observation**: `AnnotationText` is `minOccurs="0"` — every field in `AnnotationType` is optional including text.

**Decision**: `texts: Option<LocalisedString>` not `LocalisedString`.

---

### D-0002 — Reference types kept distinct

| **Area**     | Reference types |
| **Phase**    | M0 |
| **Status**   | Active |
| **Keywords** | reference-types, spec-alignment, domain-model |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) |

**Observation**: `DataStructureReference`, `CodelistReference`, `DataflowReference`, and `ConceptReference` share overlapping field sets and could be collapsed into a unified type.

**Decision**: Kept as distinct types — each maps 1-to-1 to a named reference concept in the SDMX information model.

**Rationale**: Structural repetition is accepted as the cost of spec alignment; distinct types absorb per-type field divergence naturally as the spec evolves.

---

### D-0003 — Hierarchical codes use flat mapping

| **Area**     | Codelist |
| **Phase**    | M0 |
| **Status**   | Active |
| **Keywords** | codelist, hierarchical-codes, data-structures, no_std |

**Observation**: Hierarchically nested codes could be modelled as a recursive tree (codes holding children directly) or as a flat map with parent references.

**Decision**: Flat mapping with `parent_id: Option<String>` — maps 1-to-1 with the schema representation.

**Rationale**: Avoids `Rc`/`Arc` and the associated multi-threading and serialisation complexity; consistent with the wire representation in the spec.

---

### D-0004 — Identifier validation at construction

> **Promoted to [ADR-0021](adr/0021-domain-invariant-validation-and-encapsulation-strategy.md)** (2026-06-11), consolidated with D-0005 and D-0017 as the domain invariant validation and encapsulation strategy. The ADR is now the authoritative statement; this entry is retained as the audit trail.

| **Area**     | Identifiers |
| **Phase**    | M0 |
| **Status**   | Promoted(ADR-0021) |
| **Keywords** | validation, ncname, constructor, invariants |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) |
| **Related**  | [D-0005](#d-0005), [D-0020](#d-0020), [D-0023](#d-0023) |

**Observation**: Identifier validation could live in constructors, parsers, or be deferred to query time.

**Decision**: Validated in `new()` constructor — the single write path for invariant-bearing types.

**Rationale**: Both serde-driven and streaming accumulator construction paths call it, enforcing identical validation regardless of caller.

**Consequences**: This entry settles *where* identifier validation runs (the single write path), not *which lexical rule* it applies. The original framing assumed one blanket NCName rule; that was corrected by [D-0023](#d-0023), which establishes that the rule is per-artefact (IDType for generic ids and `Code`; NCNameIDType for `Agency`/`Concept`; NestedNCNameIDType for `agencyID`). The single-write-path commitment here is unchanged.

---

### D-0005 — Invariant-bearing types use private fields and custom Deserialize

> **Promoted to [ADR-0021](adr/0021-domain-invariant-validation-and-encapsulation-strategy.md)** (2026-06-11), consolidated with D-0004 and D-0017. The ADR is now the authoritative statement; this entry is retained as the audit trail.

| **Area**     | Encapsulation |
| **Phase**    | M0 |
| **Status**   | Promoted(ADR-0021) |
| **Keywords** | serde, validation, constructor, invariants, encapsulation |
| **Source**   | [serde derive internals](https://serde.rs/derive.html) |
| **Related**  | [D-0004](#d-0004), [D-0017](#d-0017) |

**Observation**: Serde's derived `Deserialize` bypasses user-defined constructors and constructs structs directly, silently defeating constructor validation.

**Decision**: Invariant-bearing types use private fields and custom `Deserialize` impls that accumulate fields via a serde visitor and call the validated `new()` constructor at completion. Derived `Deserialize` is used only for types with no invariants.

---

### D-0006 — BTreeMap used throughout

> **SUPERSEDED 2026-06-11 by [D-0051](#d-0051)** (under [ADR-0023](adr/0023-two-layer-infoset-store-and-derived-views-architecture.md)). Keyed `BTreeMap` storage collapses element order always and silently drops schema-valid duplicate-id entries (official samples exhibit them); both are wire distinctions the Infoset Store must preserve. Wire collections are now ordered `Vec`s with first-match lookup views. The `no_std` motivation below stands (`Vec` is even more basic); the sorted-iteration determinism argument is inverted — sorting is itself a normalisation, and wire-order-out is the infoset-exact determinism. Body retained for provenance.

| **Area**     | Collections |
| **Phase**    | M0 |
| **Status**   | Superseded(D-0051) |
| **Keywords** | collections, no_std, determinism, serialisation |
| **Source**   | [ADR-0005](adr/0005-adopt-no-std-with-alloc-for-sdmx-types-and-sdmx-parsers.md) |

**Observation**: `HashMap` is unavailable in `no_std` + `alloc` environments; `BTreeMap` provides deterministic sorted iteration critical for reproducible serialized output.

**Decision**: `BTreeMap` used throughout.

**Rationale**: Satisfies the `no_std` constraint (consequence of ADR-0005), provides deterministic serialisation order, and is cache-friendly at SDMX metadata cardinalities (10–5,000 items).

---

### D-0007 — Owned String for all text fields

> **Promoted to [ADR-0022](adr/0022-owned-string-ownership-strategy.md)** (2026-06-11) as the workspace-wide API ownership commitment. The ADR is now the authoritative statement; this entry is retained as the audit trail.

| **Area**     | String ownership |
| **Phase**    | M0 |
| **Status**   | Promoted(ADR-0022) |
| **Keywords** | strings, lifetimes, no_std, domain-model |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) |

**Observation**: Domain types could use `&str`, `Cow<str>`, or owned `String` for text fields.

**Decision**: Owned `String` throughout.

**Rationale**: Keeps domain structures lifetimeless (`'static`), simplifies consumer code and client caching. Lifetime complexity is confined to parser tokenise loops.

---

### D-0008 — chrono::DateTime for date-time fields

| **Area**     | DateTime typing |
| **Phase**    | M0 |
| **Status**   | Active |
| **Keywords** | datetime, chrono, no_std, spec-alignment |
| **Spec ref** | [SDMXCommon.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXCommon.xsd#L351-L374) |
| **Source**   | [chrono docs](https://docs.rs/chrono/0.4/chrono/) |

**Observation**: `validFrom`/`validTo` are `xs:dateTime` in the spec; options were `String`, a validated `DateTimeString` newtype, or first-class `chrono::DateTime`.

**Decision**: `chrono::DateTime<FixedOffset>` with `default-features=false, features=["alloc"]`.

**Rationale**: Rejects arbitrary strings, gives correct RFC 3339 parse/format/round-trip, safe for `no_std` + headless wasm32. `time` crate rejected: its `formatting` feature requires `std`. Validated newtype rejected: defers the problem without solving it.

---

### D-0009 — isFinal removed

| **Area**     | Maintainable artefacts |
| **Phase**    | M0 |
| **Status**   | Active |
| **Keywords** | spec-alignment, maintainable, sdmx-3x |
| **Spec ref** | [SDMXCommon.xsd 3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXCommon.xsd#L390-L409) + [3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXCommon.xsd#L394-L418) |
| **Related**  | [D-0010](#d-0010) |

**Observation**: `isFinal` was present on `MaintainableType` in SDMX 2.1 but is absent from both 3.0 and 3.1 `MaintainableType`.

**Decision**: Field removed — carrying it would model a 2.1 concept with no 3.x basis.

---

### D-0010 — isPartialLanguage added

> **Amended 2026-06-11 by [D-0052](#d-0052)**: `is_partial_language` is stored as `Option<bool>` (stated-vs-absent preserved; `false` is the `effective_*()` view's default) — XSD defaulting is a view over the data, not the data itself.

| **Area**     | Maintainable artefacts |
| **Phase**    | M0 |
| **Status**   | Active (statedness storage amended by [D-0052](#d-0052)) |
| **Keywords** | spec-alignment, maintainable, sdmx-3.1, localisation |
| **Spec ref** | [SDMXCommon.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXCommon.xsd#L394-L418) |
| **Related**  | [D-0009](#d-0009) |

**Observation**: `MaintainableType` gains `isPartialLanguage: xs:boolean` (default false) in **SDMX 3.1 only** — it is absent from the 3.0 `MaintainableType` (verified: 0 occurrences in `specs/3.0/schemas/SDMXCommon.xsd`, present in 3.1). This is the same provenance class as `AvailabilityConstraint` (a 3.1 addition the canonical superset must carry).

**Decision**: `is_partial_language: bool` added to `MaintainableMetadata`; replaces `is_final` as the boolean flag on maintainable artefacts. The field is carried unconditionally as a superset member; the `false` default applies when parsing a 3.0 payload (which has no such attribute) exactly as it does for an absent 3.1 attribute.

---

### D-0011 — AnnotationURL is a vec of structs

| **Area**     | Annotation |
| **Phase**    | M0 |
| **Status**   | Active |
| **Keywords** | annotation, cardinality, spec-alignment |
| **Spec ref** | [SDMXCommon.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXCommon.xsd#L219-L255) |
| **Related**  | [D-0001](#d-0001) |

**Observation**: `AnnotationURL` is `0..*` with an optional `xml:lang` attribute; `AnnotationValue` is a non-localised string field also present on `AnnotationType`.

**Decision**: `annotation_source: Option<String>` removed (no spec basis); `annotation_urls: Vec<AnnotationUrl>` added; `annotation_value: Option<String>` added; `AnnotationUrl { url, lang }` struct defined.

---

### D-0012 — AttributeRelationship is a structured enum

| **Area**     | Data structure |
| **Phase**    | M0 |
| **Status**   | Active |
| **Keywords** | attribute, attachment, enum, spec-alignment |
| **Spec ref** | [SDMXStructureDataStructure.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureDataStructure.xsd#L192-L220) |
| **Related**  | [D-0019](#d-0019) |

**Observation**: SDMX `AttributeRelationship` is a structured choice (Dataflow / Observation / Group(id) / Dimensions(ids)), not a flat category enum.

**Decision**: Flat `AttributeAttachmentLevel` enum replaced with `AttributeRelationship` enum carrying relationship specifics. Attachment level is derivable from the variant.

**Rationale**: Domain model bakes in full structure now so parsers (Phase 2) have somewhere to put the data without information loss.

---

### D-0013 — AvailabilityConstraint carries no MaintainableMetadata

| **Area**     | Constraints |
| **Phase**    | M0 |
| **Status**   | Active |
| **Keywords** | constraint, availability, maintainable, spec-alignment |
| **Spec ref** | [SDMXStructureConstraint.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureConstraint.xsd#L240-L262) |

**Observation**: `AvailabilityConstraintType` in the spec is non-maintainable — no agencyID, no version, no registry identity. It is a response type for availability queries, not a registerable artefact.

**Decision**: `AvailabilityConstraint` retains a place in `ConstraintModel` as a domain type but carries no `MaintainableMetadata`; has `attachment`, a single `CubeRegion`, and optional `series_count`/`obs_count`.

**Rationale**: Asymmetry with `ReportingConstraint` (renamed `DataConstraint`, [D-0037](#d-0037)) is intentional — it mirrors the spec asymmetry.

---

### D-0014 — uri added to IdentifiableMetadata

| **Area**     | Identifiable artefacts |
| **Phase**    | M0 |
| **Status**   | Active (`uri` clause stands; the `Link`-omission clause corrected by [D-0035](#d-0035)) |
| **Keywords** | identifiable, uri, urn, link, superseded-in-part, spec-alignment |
| **Spec ref** | [SDMXCommon.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXCommon.xsd#L301-L327) |
| **Related**  | [D-0035](#d-0035) |

> **`Link`-omission clause corrected 2026-06-10 by [D-0035](#d-0035).** The claim below that `Link` is "a transport-layer affordance belonging in the HTTP response envelope" is **factually wrong on the schema**: `LinkType` is on `IdentifiableType` itself (`minOccurs="0" maxOccurs="unbounded"`, 3.0 and 3.1), persisted in the structure message, and carries a typed relationship + target url/urn + media-type hint — strictly more than `uri`. It is now modelled as `links: Vec<Link>` on `IdentifiableMetadata` (D-0035). The **`uri` addition below stands**; only the Link-omission is reversed. Body retained for provenance.

**Observation**: `IdentifiableType` in 3.1 carries three identity fields: `id`, `urn`, and `uri` (human-navigable URI, distinct from the machine-resolvable URN).

**Decision (Link clause superseded — see note)**: `uri: Option<String>` added to `IdentifiableMetadata`. `Link` elements (REST HATEOAS hypermedia) are omitted.

**Rationale (of the superseded Link clause)**: `Link` elements were treated as a transport-layer affordance belonging in the HTTP response envelope, not the domain model. *(Superseded:)* D-0035 shows this misread the schema — `Link` is a structure-message domain member, modelled accordingly.

---

### D-0015 — MeasureList is optional

> **Corrected by [D-0025](#d-0025)** — the *optionality* observation stands, but the cardinality decision was wrong: SDMX 3.x has *multiple* measures (`maxOccurs="unbounded"`), not a single `PrimaryMeasure`. The model is now `measures: BTreeMap<String, Measure>` (empty = measure-less), and `MissingPrimaryMeasure` was already removed. Body retained as audit trail.

| **Area**     | Data structure |
| **Phase**    | M0 |
| **Status**   | Active (cardinality clause corrected by [D-0025](#d-0025)) |
| **Keywords** | measure, dsd, optionality, spec-alignment |
| **Spec ref** | [SDMXStructureDataStructure.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureDataStructure.xsd#L81-L95) |

**Observation**: `MeasureList` is `minOccurs="0"` in the DSD `DataStructureComponentsType` — the spec permits measure-less DSDs for metadata use cases.

**Decision**: `measure: Option<PrimaryMeasure>` — domain model aligns to spec. `MissingPrimaryMeasure` error variant is live for callers that require a measure in a specific operation context.

---

### D-0016 — LocalisedString rejects blank keys (kept); blank-value rejection withdrawn under D-0031

| **Area**     | Localisation |
| **Phase**    | M0 |
| **Status**   | Active (value-rejection clause amended by [D-0031](#d-0031); store shape amended by [D-0051](#d-0051); key clauses superseded by [D-0059](#d-0059)) |
| **Keywords** | localisation, validation, xs-language, bcp47, data-quality, round-trip, no_std |
| **Spec ref** | [SDMXCommon.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXCommon.xsd#L118-L131) (`TextType` = `xs:string`); [xml.xsd](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/xml.xsd) (`xml:lang` = `xs:language`) |
| **Related**  | [D-0011](#d-0011), [D-0031](#d-0031) |

> **Amended 2026-06-11 by [D-0051](#d-0051)**: the store is an ordered list of `(language, text)` entries in wire order (duplicate language tags are schema-valid and preserved; `get(lang)` is a first-match view, `first()` is first-in-wire-order). The non-empty and non-blank-key invariants below are unchanged.
>
> **Key clauses superseded 2026-06-11 by [D-0059](#d-0059)**: the language key now stores statedness (`Option<String>` — `TextType` declares `xml:lang` `default="en"`, a missed D-0052 class) AND is held verbatim even when blank or off-pattern (the parsable-within-spec principle; key validity is a catalogued lint). The blank-key rejection below is withdrawn and `MalformedLocalisation` removed with its producer; only the non-empty invariant survives.
>
> **REVISED 2026-06-10 under D-0031, and a factual correction.**
> 1. **Value-rejection WITHDRAWN.** The original rejected a blank/whitespace-only *value*. `TextType` is bare `xs:string`, so a blank value is **mechanically schema-valid** — the rejection traded strict spec adherence for consumer ergonomics by collapsing input for a data-quality reason, an architectural violation D-0031 forbids. Blank values are now **stored verbatim** (round-trippable); "a name with no visible text is dubious" is a non-destructive **lint**, not a `new()` error.
> 2. **Key-rejection STANDS — with a corrected basis.** The original Observation claimed the spec "does not constrain language key format." That is **wrong**: the key is `xml:lang` = `xs:language` ([xml.xsd](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/xml.xsd)), whose pattern (`[a-zA-Z]{1,8}(-[a-zA-Z0-9]{1,8})*`) requires a leading letter token. A blank key is therefore **mechanically schema-invalid**, so rejecting it enforces the exact XSD constraint (Layer-1 legitimate), not an invented constraint — it survives D-0031 for the right reason. The check remains *under*-strict (blank-only, not full `xs:language`), which is round-trip-safe (accepts a superset, never rejects a valid tag); tightening to full `xs:language` would also be mechanically exact (off-pattern is mechanically invalid) but stays a deferred parser-layer concern.
>
> Net: `LocalisedString::new()` rejects an empty map (schema-invalid: ≥1 Name required) and a blank key (schema-invalid `xs:language`); it no longer inspects values. The original body is retained below for provenance.

**Observation (original — note the key-format claim is corrected above)**: The spec (`TextType` = `xs:string`) permits empty-string values (`{"en": ""}`) and does not constrain language key format — both are valid on the wire. A whitespace-only value (`{"en": "   "}`) or key is exactly as unresolvable as an empty one: it carries no name and cannot serve as (or round-trip through) a meaningful `xml:lang` tag.

**Decision (value clause superseded — see note)**: Domain model rejects, for every entry, a key that is empty *or whitespace-only* (not a valid BCP 47 tag; unresolvable) and a value that is empty *or whitespace-only* (semantically equivalent to absent). The whitespace test is `s.chars().all(char::is_whitespace)`. `EmptyLocalisation` covers the no-entries case; `MalformedLocalisation(String)` carries the offending key so callers can distinguish the two failure modes.

**Rationale (of the superseded value clause)**: These are data quality failures the domain model should not carry silently. *(Superseded for values:)* D-0031 holds that a *schema-valid* shape is never refused for a quality reason — the quality concern moves to a lint. The argument still holds for the *key*, but the operative reason is now "blank key is schema-invalid `xs:language`," not "data quality."

**Consequences**: (1) **trimming** remains out of scope (it always was, and is moot for values now). (2) `MalformedLocalisation` is now only raised for keys. (3) The blank-value lint is the first member of the coherence-lint surface D-0031 introduces (alongside the `position`-consistency and `isExternalReference`+URL lints).

---

### D-0017 — Field visibility rule

> **Promoted to [ADR-0021](adr/0021-domain-invariant-validation-and-encapsulation-strategy.md)** (2026-06-11), consolidated with D-0004 and D-0005. The ADR is now the authoritative statement; this entry is retained as the audit trail.
>
> **Normalisation clause amended by [D-0026](#d-0026).** The field-*visibility* rule below stands and `CubeRegion` still keeps public fields. But the specific "normalise empty value-set → absent" mechanism described here is **withdrawn**: the remodel (D-0026) showed a component-with-no-values is a *meaningful distinct state*, so collapsing it would erase information — the normalisation was not merely unnecessary but semantically wrong. The `BTreeSet` value model it referenced no longer exists.

| **Area**     | Encapsulation |
| **Phase**    | M0 |
| **Status**   | Promoted(ADR-0021) (normalisation clause amended by [D-0026](#d-0026)) |
| **Keywords** | encapsulation, invariants, field-visibility, domain-model |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §7 |
| **Related**  | [D-0005](#d-0005), [D-0019](#d-0019), [D-0026](#d-0026) |

**Observation**: Some domain types own an invariant that mutation could break (`LocalisedString` — empty map illegal; `ItemScheme<I>` — map key must equal `item.id()`; `IdentifiableMetadata` — id must be a valid identifier). Others are transparent data carriers where every field combination is coherent (`CubeRegion` — any pairing of its fields is valid).

**Decision**: Private fields where mutation could break an invariant the type is responsible for; public fields where the type is a transparent carrier and any invariant lives in the constructor or deserializer.

**Rationale**: Encode in the type what the type can actually protect; document (and test) what it cannot. `CubeRegion` keeps public fields — it owns no cross-field invariant. (The original rationale cited an empty-value-set *normalisation* in its custom `Deserialize`; that clause is withdrawn per the blockquote above — D-0026. The custom `Deserialize` survives for *structural* reasons, mapping the two wire collections, not for normalisation.) `include: bool` stays `bool` not an enum — it is a transparent flag, not an ambiguous state.

---

### D-0018 — bool vs enum chosen by call-site visibility

| **Area**     | Conventions |
| **Phase**    | M0 |
| **Status**   | Active |
| **Keywords** | bool, enum, conventions, ergonomics, domain-model |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) |
| **Related**  | [D-0012](#d-0012), [D-0017](#d-0017) |

**Observation**: A two-valued domain axis can be modelled as `bool` or as a two-variant enum. The "make illegal states unrepresentable" design pattern does not completely decide it — a `bool` has no illegal state; both values are meaningful and mutually exclusive.

**Decision**: Use an enum when the value appears without its field name in view (function arguments, return values, tuple positions). Use `bool` when the value is always accessed through a self-naming field. The deciding test is call-site visibility, not state count.

**Rationale**: `Attribute::new(.., Usage::Mandatory)` is self-documenting where `(.., true)` is opaque. Conversely, `region.include` already names the axis, so an enum would add a type, a match arm, and import noise without disambiguating anything. `Usage` passes the test (set positionally in a constructor — for both `Attribute` and `Measure`); `CubeRegion.include` fails it (always reached through its named field). (The enum was renamed `AttributeUsage`→`Usage` in D-0025, since the spec's single `UsageType` serves both attributes and measures.)

---

### D-0019 — AttributeRelationship data variants wrap validating newtypes

> **`DimensionIds` renamed 2026-07-08:** `DimensionIds` → `DimensionRefs` — the newtype holds `DimensionRef`s, not ids, and joins the `…Refs` collection family (`DataStructureRefs`/`DataflowRefs`). A name sync, not a new decision, so the references below are updated in place.
>
> **`GroupId` clause amended 2026-07-08 by [D-0077](#d-0077).** `GroupId::new()` now validates the full `IDType` grammar, not merely non-empty; `Error::EmptyGroupId` is removed (the empty lexeme reports `InvalidIdentifier`, like every identifier site). `DimensionRefs`' non-empty list invariant and the `group()`/`dimensions()` forwarders stand; `DimensionRef` itself is separately promoted by D-0077.

| **Area**     | Data structure |
| **Phase**    | M0 |
| **Status**   | Active (GroupId clause amended by [D-0077](#d-0077)) |
| **Keywords** | attribute, enum, newtypes, invariants, unrepresentable |
| **Spec ref** | [SDMXStructureDataStructure.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureDataStructure.xsd#L192-L220) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.6 |
| **Related**  | [D-0005](#d-0005), [D-0012](#d-0012), [D-0017](#d-0017) |

**Observation**: The data-carrying variants `Group(..)` and `Dimensions(..)` can hold structurally meaningless values: an empty group id, or an empty dimension list. A bare `pub` variant leaves `Dimensions(vec![])` freely constructible and lets derived `Deserialize` build it from the wire (the D-0005 gap).

**Decision**: Data variants wrap private-field newtypes (`GroupId`, `DimensionRefs`) whose validating `new()` rejects empty — so the invalid state is unrepresentable. Ergonomic forwarders `AttributeRelationship::group()/dimensions()` wrap the newtype constructors. Unit variants (`Dataflow`, `Observation`) stay freely constructible — they carry no data and cannot be invalid.

**Rationale**: Chosen over enforcing in `Attribute::new()`, which would be a layer violation — the type that owns the invariant enforces it. Newtypes carry custom `Deserialize`; the enum and `Attribute` ride on derived `Deserialize` per the D-0017 cross-field rule.

---

### D-0020 — Identifiers validated at declaration, not at reference

> **Reference-lexeme clause amended 2026-07-08 by [D-0077](#d-0077).** The "only structural well-formedness is enforced" clause is withdrawn: local reference identifiers now validate their lexical tier at construction (lexical grammar is a local property of the lexeme, independent of resolution — the wire validates a reference lexeme whether or not its target exists). The boundary this entry draws narrows to referential integrity (does the id name a declared component?), which stays deferred exactly as the Consequences record.

| **Area**     | Identifiers |
| **Phase**    | M0 |
| **Status**   | Active (reference-lexeme clause amended by [D-0077](#d-0077)) |
| **Keywords** | validation, ncname, identifiers, referential-integrity |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) |
| **Related**  | [D-0004](#d-0004), [D-0019](#d-0019) |

**Observation**: An identifier string appears in two roles: as an **identity** (the canonical declaration — `IdentifiableMetadata.id`) and as a **reference/pointer** to an identity declared elsewhere (the embedded ids in `AttributeRelationship`, `parent_id`, and the reference structs). Lexical NCName validity is a local, checkable property in both roles.

**Decision**: NCName-validate identifiers at the point of declaration (identity) only. Embedded reference ids are not NCName-validated — only structural well-formedness is enforced (non-empty, per [D-0019](#d-0019)).

**Rationale**: Validating a pointer is redundant if the target was validated when minted, and insufficient if it was not — a lexically-valid id that points nowhere is still broken.

**Consequences**: Referential integrity (does this id name a component that actually exists in the DSD?) is deferred to a higher-level validation pass — parser or a future DSD-integrity check. This explains why the reference structs' string fields are unvalidated, not merely that they are.

---

### D-0021 — #[non_exhaustive] per public enum, not blanket

| **Area**     | Conventions |
| **Phase**    | M0 |
| **Status**   | Active |
| **Keywords** | enums, non-exhaustive, semver, api-stability |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.6, §5.8, §5.9 |

**Observation**: A public enum either has a known, routine growth path or a bounded, spec-fixed variant set. Reflexively stamping `#[non_exhaustive]` everywhere "for safety" imposes overhead on every consumer with a permanent catch-all `match` arm to provide flexibility a bounded enum will never use.

**Decision**: `#[non_exhaustive]` where variant growth is routine and spec-documented; exhaustive where the set is bounded and a new member is a rare, significant event.

**Rationale**: Non-exhaustive lets routine spec-completion additions land without breaking consumers. Exhaustive lets consumers write a complete `match` with no catch-all, and makes a genuinely new variant a deliberate MINOR-bump breaking change that surfaces (compile error in downstream matches) rather than being silently absorbed.

**Consequences**: Applied — originally `ConstraintAttachment` was the sole non-exhaustive enum (spec listed future targets); **D-0034 superseded that** by splitting it into two *bounded, exhaustive* per-constraint attachment enums, so as of D-0034 **no modelled enum is `#[non_exhaustive]`** — the policy stands for any future routine-growth enum but currently has no instance. `Error`, `ConstraintModel`, `AttributeRelationship`, `Usage`, and now `DataConstraintAttachment`/`AvailabilityConstraintAttachment` are exhaustive (bounded spec-fixed sets; `MissingPrimaryMeasure` was removed rather than reserved — it rejoins on a MINOR bump when its operation lands). (`Usage` = the renamed `AttributeUsage`, D-0025.)

---

### D-0022 — Round-trip fidelity is semantic, not byte-level

| **Area**     | Serialisation |
| **Phase**    | M0 |
| **Status**   | Superseded(D-0031) (canonicalise-in-store → Infoset Store + derived view; residual clause by [D-0052](#d-0052)) |
| **Keywords** | round-trip, serialisation, canonical-model, position, superseded, spec-alignment |
| **Spec ref** | [SDMXStructureDataStructure.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureDataStructure.xsd#L344-L364) (`BaseDimensionType.position`) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §4, §5.6, §7; [ADR-0008](adr/0008-model-sdmx-3-0-and-3-1-divergence-with-a-unified-constraintmodel.md) |
| **Related**  | [D-0017](#d-0017), [D-0031](#d-0031) |

> **Residual "semantic, not byte-level" clause RETIRED 2026-06-11 by [D-0052](#d-0052)** (under [ADR-0023](adr/0023-two-layer-infoset-store-and-derived-views-architecture.md)): defining round-trip equality "as domain values" is circular — whatever the model collapses becomes "not semantic" by definition. The contract is now stated wire-side: equivalence up to the format's own non-information layer (C14N-class lexical accidents), with order, duplicates, statedness, and lexemes round-tripping exactly.
>
> **SUPERSEDED 2026-06-10 by D-0031.** This decision *canonicalised* `Dimension.position` into a mandatory `u32`, deliberately collapsing the wire's absent-vs-stated distinction so an omitting input and a stating input "converge on the same model." D-0031 forbids exactly that — collapsing a schema-valid wire distinction in the store for convenience. **Revised:** `position` is stored verbatim as `Option<i32>` (`None` = omitted, `Some(n)` = stated; `i32` mirrors `xs:int`); `DataStructureDefinition::new()` no longer sorts or canonicalises by it. The spec's "*if specified must be consistent with* the key-descriptor position" is stated **only in prose** (`<xs:documentation>`), so a DSD with a contradictory stated position still validates against the XSD — under D-0031 that is schema-valid input the store must hold, and the consistency rule becomes a non-destructive **lint**, not a `new()` rejection. The canonical/effective position is a **view**: `Dimension::effective_position(list_index)` returns the stated value or, when absent, the list index. D-0022's underlying observation (position is derivable from order) stands; only its *resolution* (collapse in store) is replaced (derive in view). The original body is retained below for provenance.

**Observation**: The spec marks some fields optional on the wire because they are derivable, not because they are semantically absent — e.g. `Dimension.position` (`xs:int`, optional; derivable from `DimensionList` order). Modelling such a field as `Option`/raw-mirror would carry the wire's explicit-vs-implicit distinction into the domain model.

**Decision (superseded — see note)**: The domain model canonicalises derivable-optional wire fields to a single mandatory in-memory form. The round-trip guaranteed is semantic (`parse(serialize(x))` equals `x` as domain values), not byte-level. `Dimension.position` is made mandatory `u32` and canonicalised on construction.

**Rationale (of the superseded decision)**: An input that omitted `position` and one that stated it converge on the same model, consistent with `CubeRegion` empty-set normalisation (D-0017). *(Superseded:)* D-0031 holds that this convergence is precisely the information loss to avoid — the convenience belongs in a view, not the store.

**Consequences**: Under D-0031 the open Phase-2 writer decision *resolves cleanly*: the store records what the wire said (`Option`), so the writer emits `position` iff it was present (`Some`) — verbatim round-trip, no always-emit-vs-never-emit policy call needed. (D-0017's empty-set normalisation, referenced above, was separately withdrawn by D-0026.)

---

### D-0023 — Identifier validation is per-artefact lexical type, not blanket NCName

> **Reference clauses amended 2026-07-08 by [D-0077](#d-0077)**: consequence (2)'s "structural-only at reference" half is withdrawn — reference ids and stated `Parent` fields now validate their lexical tier at construction, the same per-site tier system extended to reference sites (where the tier diverges between editions, the union: `Code.parent_id` validates `IDType`). The "`Code` stays a derive-only carrier" clause is superseded with it: `Code` crosses to an invariant-bearing type whose `new()` validates the stated `parent_id`. The tier table, the validators, and every declaration-side placement stand.
>
> **Variant renamed 2026-07-08:** `Error::InvalidAgencyIdentifier` → `Error::InvalidNestedNcNameIdentifier`, so the name matches the lexical tier it reports (`NestedNCNameIDType`) rather than the `agencyID` field that was once its only producer; the nested constraint selection-node ids report it too. A name sync, not a new decision, so the reference below is updated in place.
>
> **Amended 2026-06-11 by [D-0052](#d-0052)**: the `fixed="AGENCIES"` *value* is now enforced — a stated value differing from a fixed value is mechanically schema-invalid — so `AgencyScheme::new()` is **fallible** (`FixedAttributeMismatch`), not infallible as stated below. The lexical-tier system itself is unchanged.

| **Area**     | Identifiers |
| **Phase**    | Phase-1 |
| **Status**   | Active (fixed-value enforcement amended by [D-0052](#d-0052); reference clauses amended by [D-0077](#d-0077)) |
| **Keywords** | validation, ncname, idtype, identifiers, spec-alignment, no_std |
| **Spec ref** | [SDMXCommonReferences.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXCommonReferences.xsd#L1572-L1579) (`IDType`, `NCNameIDType`, `NestedNCNameIDType`); [SDMXStructureBase.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureBase.xsd#L43-L58) (`ItemBaseType.id`); [SDMXStructureOrganisation.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureOrganisation.xsd#L229-L250) (`AgencyType.id`); [SDMXStructureConcept.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureConcept.xsd#L50-L77) (`ConceptBaseType.id`) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.2, §5.5, §7 |
| **Related**  | [D-0004](#d-0004), [D-0005](#d-0005), [D-0020](#d-0020) |

**Observation**: SDMX does **not** use one identifier lexical type. The XSDs use three, and the generic/`Code` id is the *loosest*:

| Where the id appears | Spec type | Pattern | Allows |
|---|---|---|---|
| Generic id; `Code` id; `Code`/`Item` `Parent`; **DSD / Dataflow / DataConstraint / AgencyScheme** ids | `IDType` | `[A-Za-z0-9_@$\-]+` | leading digit, `@`, `$` |
| `Agency`, `Concept` ids (+ `Parent`); **component ids** (`Dimension`, `Attribute`, `Measure`); **`Codelist`, `ConceptScheme`** scheme ids | `NCNameIDType` | `[A-Za-z][A-Za-z0-9_\-]*` | — (strict NCName) |
| `agencyID` (the `agency` field) | `NestedNCNameIDType` | `[A-Za-z][A-Za-z0-9_\-]*(\.[A-Za-z][A-Za-z0-9_\-]*)*` | dotted NCName segments |

**Maintainable artefacts are NOT uniform** — the spec restricts some scheme ids to NCName and leaves others at the generic `IDType`. Verified per type: `Codelist`/`ConceptScheme` restrict id to `NCNameIDType` (their ids become simple-type names in structure-specific schemas — `CodelistBaseType`/`ConceptSchemeBaseType` in the structure XSDs); `DataStructure`/`Dataflow`/`DataConstraint` extend `MaintainableType` *without* restriction, inheriting `IDType`; `AgencyScheme` is `IDType` with `fixed="AGENCIES"`. So `Codelist::new()`/`ConceptScheme::new()` re-validate NCName (fallible, custom `Deserialize`); `AgencyScheme::new()` stays infallible + derived. The asymmetry is the spec's, not an oversight — do not "consistency-fix" AgencyScheme to NCName (`"AGENCIES"` is a valid NCName anyway, so the rule follows the declared type, not the fixed value).

Components (`Dimension`/`Attribute`/`Measure`) carry `NCNameIDType` ids because `ComponentBaseType` restricts the id (component ids become XML element/attribute names) — verified in `specs/3.1/schemas/SDMXStructureBase.xsd`. They are validated-item types exactly like `Agency`/`Concept`. Note: component id is also `use="optional"` (inherit the concept id if absent); that **inheritance** is a separate derivable-optional decision, deliberately not built here — see D-0025.

A blanket `validate_ncname()` on every id therefore **rejects valid SDMX** — a code id of `1`, `_T`, `EUR$`, or `@INTERNAL` is legal `IDType` but fails NCName — and is only incidentally correct for `Agency`/`Concept`.

**Decision**: Validate each identifier against its own lexical type via three hand-rolled (`no_std`, no regex crate) validators: `validate_id` (IDType), `validate_ncname` (NCNameIDType), `validate_nested_ncname` (NestedNCNameIDType). `IDType` is enforced with the full character-set regex (not merely non-empty). Placement: `IdentifiableMetadata::new()` runs `validate_id` (the loosest tier, shared by all identifiable artefacts); `MaintainableMetadata::new()` becomes **fallible** and runs `validate_nested_ncname` on `agency`; `Concept` and `Agency` are **promoted** from derive-only carriers to invariant-bearing types (private fields, validated `new()` that re-checks their own id via `validate_ncname`, custom `Deserialize`, trait delegation). `Code` stays a derive-only carrier — `IDType` from the base check suffices. `Error` gains `InvalidNcNameIdentifier(String)` and `InvalidNestedNcNameIdentifier(String)`; the existing `InvalidIdentifier` message is reworded from NCName to IDType.

**Rationale**: The type that knows its own spec lexical type owns the stricter check (mirrors [D-0019](#d-0019): the type owning the invariant enforces it). The base IDType check stays for everyone even on `Concept`/`Agency` — it is harmless redundancy (every NCName is a valid IDType) and avoids reintroducing an `IdKind` parameter or an unchecked metadata constructor. Two-layer error ordering is deliberate: an `@`-id `Concept` reports `InvalidIdentifier` (IDType, fired first by the base) while a `1abc`-id `Concept` reports `InvalidNcNameIdentifier` (NCName, fired by `Concept::new()`).

**Consequences**: (1) `Concept`/`Agency` cross the §7 carrier→invariant-bearing line; the §5.5 narrative now presents *two* item patterns (carrier: `Code`; validated-item: `Concept`/`Agency`), not one. (2) Reference ids and `Parent` fields are unaffected — [D-0020](#d-0020) still governs them (validate at declaration, structural-only at reference). (3) The `agency`-field validation makes `MaintainableMetadata::new()` fallible, changing the design's earlier "its `new()` is infallible" claim. (4) Promoting `Concept` forces its struct body to be written out, incidentally closing the "Concept body never shown" documentation gap — but **not** the separate question of whether `Concept` should carry `CoreRepresentation` (a deferred superset decision, tracked outside this entry).

---

### D-0024 — version is optional (`Option<SdmxVersion>`); un-versioned is distinct from 1.0

> **Validation/naming clauses amended by [D-0027](#d-0027).** The `Option` decision, the un-versioned-≠-1.0 rationale, and the `VersionDisplay`/`<unversioned>` mechanism all STAND. But the **Tier-A grammar-deferral** clause is **withdrawn** — `SdmxVersion::new()` now validates the full `VersionType` grammar (D-0027 supersedes the "defer to parser" stance) and retains the parsed decomposition (`major`/`minor`/`patch`/`extension`). The type is renamed `Version` → `SdmxVersion` (D-0027 naming rule).
>
> **Confirmed compliant with [D-0031](#d-0031) (no change).** This decision is a strict application of the two-layer principle, decided before it was named: the store is verbatim (`Option<SdmxVersion>` preserves un-versioned vs every value; `SdmxVersion.raw` is the verbatim lexeme, never normalised — `1.3` and `1.03` stay distinct strings, D-0027), the defaulting *collapse* (absent→`1.0`) was explicitly **rejected** as lossy, and the convenience (`version_display()`, `major()`/`minor()`/`is_legacy()`) is provided as **views** over that store. Nothing to sweep — listed here only so the audit trail shows D-0031 was checked against it.

| **Area**     | Versionable artefacts |
| **Phase**    | Phase-1 |
| **Status**   | Active (validation/naming clauses amended by [D-0027](#d-0027)) |
| **Keywords** | version, optionality, newtype, spec-alignment, round-trip, no_std |
| **Spec ref** | [SDMXCommon.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXCommon.xsd#L351-L374) (`VersionableType.version` `use="optional"`); [SDMXCommonReferences.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXCommonReferences.xsd#L1608-L1613) (`VersionType`) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.2, §5.3 |
| **Related**  | [D-0008](#d-0008), [D-0016](#d-0016), [D-0022](#d-0022), [D-0027](#d-0027), [D-0031](#d-0031) |

**Observation**: `VersionableType.version` is `use="optional"` and the spec states "if not supplied, artefact is considered to be **un-versioned**." There is **no** `default="1.0"` in the XSD — un-versioned is a distinct semantic state, not a synonym for `1.0`. `VersionType` itself is a real constrained type (a union of `SemanticVersionNumberType` and `LegacyVersionNumberType`).

**Decision**: Model `version: Option<SdmxVersion>` on `VersionableMetadata`, where `SdmxVersion` is a validating newtype (D-0027). `None` means un-versioned and is preserved losslessly (distinct from `Some("1.0")`). Defaulting absent→`"1.0"` was rejected: it would collide two distinct wire states into one model state — lossy, and contrary to the canonical-superset principle. `version()` returns `Option<&SdmxVersion>`. *(The original entry deferred `SdmxVersion`'s grammar to the parser — Tier A; that was withdrawn by D-0027, which validates the full `VersionType` grammar at construction and retains the parsed decomposition.)*

**Rationale**: A standard `Option` is correct because versioning genuinely *is* optional in SDMX. The display-formatting risk (someone calling `.to_string()` without handling `None`) is closed *structurally*: `Version`'s own `Display` is verbatim (raw string, no sentinel), and the un-versioned sentinel lives only on a separate adapter `VersionDisplay<'a>(Option<&'a Version>)` reached through the `VersionableArtefact::version_display()` **default trait method**. The sentinel is `<unversioned>` — the angle brackets are outside every SDMX id/version lexical set, so it is **un-roundtrippable by design**: if it ever reaches a writer it fails validation loudly rather than passing as a valid version. The default-method placement means delegating structs inherit the display path for free (no per-impl boilerplate).

**Consequences**: (1) `VersionableArtefact::version()` changes from `-> &str` to `-> Option<&Version>`, rippling through every delegating `version()` impl (§5.4/§5.5) — they move together. (2) Writers must match on `version()` directly and emit nothing when `None`; `version_display()` is display/logging-only. (3) `valid_from`/`valid_to` are unaffected (already `Option`). (4) **Out of scope for this pass:** the `version: String` field on the reference structs (`DataStructureReference`/`CodelistReference`/`DataflowReference`) is left a plain `String` — they are references (D-0020 territory, transparent carriers), and whether they should adopt `Version`/`Option<Version>` is a separate follow-up, not part of the identifier/version remit settled here. Flagged so the `String` there reads as deliberate, not an oversight.

---

### D-0025 — DSD has multiple measures (3.x), not a single PrimaryMeasure (2.1)

> **Further amended 2026-06-11 by [D-0051](#d-0051)/[D-0052](#d-0052)**: the keyed measure map became an ordered `Vec<Measure>` (wire order and duplicates preserved; lookup is a view), and `usage` is stored as `Option<Usage>` (the schema default `optional` is an effective view).
>
> **Component-id deferral (consequence 2) superseded 2026-06-11 by [D-0057](#d-0057)**: the inheritance is now built — components store `id: Option<String>` (`ComponentMetadata`), the trait `id()` is the effective view, and `stated_id()` is the raw accessor.
>
> **Amended by [D-0049](#d-0049).** The multi-measure model, the keyed map, and the shared `Usage` all **stand**, but the "measure-less DSD = the empty map, **no `Option` wrapper**" clause is withdrawn: with the component descriptors modelled as identifiable structs (D-0049), an *absent* `MeasureList` and a *present* one are distinguishable wire states (a present descriptor carries its own annotations/links/urn and mechanically requires ≥1 `Measure`), so the DSD now holds `measure_list: Option<MeasureList>` with the keyed map inside the descriptor — and likewise `attribute_list: Option<AttributeList>`. `None` ⟺ the wire's absent list.

| **Area**     | Data structure |
| **Phase**    | Phase-1 |
| **Status**   | Active (no-Option clause amended by [D-0049](#d-0049); measure store amended by [D-0051](#d-0051), [D-0052](#d-0052); component-id deferral superseded by [D-0057](#d-0057)) |
| **Keywords** | measure, dsd, cardinality, usage, spec-alignment, component |
| **Spec ref** | [SDMXStructureDataStructure.xsd 3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXStructureDataStructure.xsd#L479-L496) + [3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureDataStructure.xsd#L491-L508) (`MeasureListType`, `MeasureType`, `UsageType`); [SDMXStructureBase.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureBase.xsd#L151-L168) (`ComponentBaseType` id = `NCNameIDType`) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §4, §5.6 |
| **Related**  | [D-0015](#d-0015), [D-0018](#d-0018), [D-0023](#d-0023) |

**Observation**: `MeasureList` contains `Measure` with `maxOccurs="unbounded"` in **both 3.0 and 3.1** — there is no single `PrimaryMeasure` element in 3.x (that was the SDMX 2.1 model). `MeasureType` carries a `usage` attribute (`UsageType` = `mandatory`|`optional`, default `optional`) — the *same* `UsageType` the spec uses for attributes. The earlier `measure: Option<PrimaryMeasure>` (D-0015) therefore could not represent a conformant multi-measure 3.x DSD — a superset-exactness defect.

**Decision**: Replace `PrimaryMeasure` with `Measure { metadata, concept, usage }` and model the DSD field as `measures: BTreeMap<String, Measure>` keyed by measure id (mirrors `attributes`; measure-less DSD = empty map, no `Option` wrapper). `AttributeUsage` is renamed `Usage` and shared by `Attribute` and `Measure` (one spec `UsageType`, one domain type). `Measure` is keyed (not `Vec`) because measures carry no coordinate-order significance, unlike `dimensions`.

**Rationale**: A keyed map matches the spec's identity-by-id and gives O(log N) lookup; an empty map is the natural "measure-less" state, consistent with how `attributes` represents "no attributes". Sharing `Usage` follows the spec, which defines one `UsageType` for both.

**Consequences**: (1) **Component NCName fold-in (D-0023 extension):** all components — `Dimension`, `Attribute`, `Measure` — have `NCNameIDType` ids per `ComponentBaseType`, so they are promoted to validated-item types (private fields, validated `new()` re-checking the id via `validate_ncname`, custom `Deserialize`). `Dimension` exposes a `position()` accessor since `DataStructureDefinition::new()` reads it to sort. (2) **Component-id inheritance deferred:** component id is `use="optional"` (inherit the `ConceptReference` id when absent). That inheritance is a derivable-optional canonicalisation (cf. `position` D-0022, `version` D-0024) deliberately **not** built in this pass — `new()` validates the id it is given; inheritance is a flagged follow-up. (3) D-0015 is superseded in part (optionality observation stands; single-measure decision withdrawn). (4) `ConceptRole` and `LocalRepresentation` on `MeasureType` are not modelled here — same Phase-1 scoping cut as elsewhere (representation is deferred; cf. the open `Concept.CoreRepresentation` question).

---

### D-0026 — CubeRegion modelled to full spec structure (dimension/component, cascade, time range)

| **Area**     | Constraints |
| **Phase**    | Phase-1 |
| **Status**   | Active (no-per-selection-include claim corrected by [D-0038](#d-0038); selection stores amended by [D-0051](#d-0051), [D-0052](#d-0052); `TimeRange` shape amended by [D-0064](#d-0064)) |
| **Keywords** | cube-region, constraint, superset, cascade, time-range, spec-alignment |
| **Spec ref** | [SDMXStructureConstraint.xsd 3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXStructureConstraint.xsd#L470-L485) + [3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureConstraint.xsd#L385-L400) (`CubeRegionType`, `RegionType`, `CubeRegionKeyType`, `ComponentValueSetType`, `MemberSelectionType`, `SimpleComponentValueType`, `TimeRangeValueType`); [SDMXCommon.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXCommon.xsd#L1206-L1208) (`CascadeSelectionType`) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.8 |
| **Related**  | [D-0013](#d-0013), [D-0017](#d-0017), [D-0018](#d-0018), [D-0022](#d-0022), [D-0038](#d-0038) |

> **Amended 2026-06-11 by [D-0051](#d-0051)/[D-0052](#d-0052)**: the two selection collections are ordered `Vec`s with ids carried on the node structs (not id-keyed maps), and the region-level `include` and per-value `cascade` are stored as `Option` (statedness; the schema defaults are effective views).
>
> **CORRECTED 2026-06-11 by [D-0038](#d-0038).** This entry's closing claim — "there is NO per-selection `include` - is itself **incorrect**: the verification looked only at `RegionType`. `include` (`xs:boolean`, optional, default `true`) is declared on **`MemberSelectionType`** ([3.1 line ~314](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureConstraint.xsd#L293-L326); 3.0 line ~402; identical both versions) and is inherited by BOTH `CubeRegionKeyType` and `ComponentValueSetType` (neither restriction prohibits it). So `include` exists at the region level **and** the selection level, and the "correction" recorded here was the error. D-0038 models the selection level (wrapper structs `CubeRegionKey`/`ComponentValueSet`), together with two further `MemberSelectionType` attributes this entry's audit also missed (`removePrefix`, both selection kinds; `validFrom`/`validTo`, KeyValue side only) and non-empty `Values` enforcement. The structural model below (two selection kinds, cascade, time range, `Empty`) otherwise **stands**; only the include claim is corrected. Body retained for provenance.
>
> **CORRECTED 2026-06-19 by [D-0064](#d-0064).** The `TimeRange = Before | After | Between { from, to }` shape recorded in the Decision below is incomplete and is superseded: `TimeRangeValueType` carries its own `validFrom`/`validTo` (`StandardTimePeriodType`) attributes, which this entry's model omitted, and the endpoints are renamed `{ start, end }` to match the schema's `StartPeriod`/`EndPeriod`. D-0064 remodels it as `TimeRange { kind, valid_from, valid_to }`. The rest of the CubeRegion structure below stands; only the `TimeRange` shape is amended. Body retained for provenance.

**Observation**: The earlier `CubeRegion { values: BTreeMap<String, BTreeSet<String>>, include: bool }` could not represent four distinctions the spec draws (identical in 3.0 and 3.1, verified):

1. **Dimension (`KeyValue`) vs attribute/measure (`Component`) selections** — the spec gives these *different types* (`CubeRegionKeyType` vs `ComponentValueSetType`) with *different id grammars* (`SingleNCNameIDType` vs `NestedNCNameIDType`) and *different cardinalities* (KeyValue value-choice mandatory; Component value-choice `minOccurs="0"`).
2. **`cascadeValues`** per value (`CascadeSelectionType` = `boolean | "excluderoot"`) — include child codes in a simple hierarchy.
3. **`TimeRange`** as an alternative to a value list (before/after/between periods).
4. The **"component referenced with no values"** state — a real selection ("present/absent regardless of value"), NOT a synonym for "all values".

It also claimed a *per-value-set* `include` which was **incorrect** — `include` is region-level only (`RegionType`), there is no per-selection include in `CubeRegion`.

**Decision**: Model the full structure. `CubeRegion { key_values: BTreeMap<String, KeyValueSelection>, components: BTreeMap<String, ComponentSelection>, include: bool }`. `KeyValueSelection = Values(Vec<CubeValue>) | TimeRange(TimeRange)` (no `Empty` — dimension selections are mandatory, so dimension-empty is unrepresentable). `ComponentSelection = Values | TimeRange | Empty` (the `Empty` variant is the components-only no-values state). `CubeValue { value: String, cascade: Cascade }`; `Cascade = None | IncludeChildren | ExcludeRoot`. `TimeRange = Before | After | Between { from, to }` over `TimePeriodRange { period, inclusive }`.

**Rationale**: Separate `KeyValueSelection`/`ComponentSelection` types (rather than one shared enum) because the spec distinguishes them at the type level — different id grammars *and* cardinalities — so a single type would be wrong for one of the two and would let a spec-invalid dimension-empty be represented. This is the benchmark-impl, make-illegal-states-unrepresentable choice (cf. [D-0019](#d-0019)). `Cascade` is an enum not a bool because the spec axis is tri-state (`boolean | "excluderoot"`), per [D-0018](#d-0018).

**Consequences**: (1) **[D-0017](#d-0017) normalisation clause withdrawn** — `ComponentSelection::Empty` is meaningful, so the old empty-set→absent normalisation would erase it; the visibility rule (public fields) survives, and the custom `Deserialize` remains only for *structural* two-collection mapping. (2) **[D-0022](#d-0022) round-trip is unaffected and in fact improved** — `Empty` and "key absent" are now distinct in-memory states that re-serialize differently, preserving a distinction the old normalisation destroyed; semantic round-trip still holds. (3) `BTreeSet` is no longer used by any domain type (import dropped). (4) Phase-1 scoping cut retained: `TimePeriodRange.period` is the string form of `StandardTimePeriodType` — full time-period typing (and the `TimeRange` endpoint operators beyond inclusive/exclusive) are deferred to the parser layer, consistent with the `version`/grammar deferrals elsewhere.

---

### D-0027 — Validated lexical newtypes (lossless raw + retained discriminant)

| **Area**     | Lexical types |
| **Phase**    | Phase-1 |
| **Status**   | Active (SdmxVersion raw clause amended by [D-0070](#d-0070); reference-version follow-up resolved by [D-0073](#d-0073)) |
| **Keywords** | newtype, validation, lossless, no_std, decimal, integer, version, time-period, naming |
| **Spec ref** | [SDMXCommon.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXCommon.xsd#L499-L504) (`StandardTimePeriodType`, `ObservationalTimePeriodType`); [SDMXCommonReferences.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXCommonReferences.xsd#L1608-L1613) (`VersionType`); W3C XSD (`xs:decimal`, `xs:integer`); [semver.org §11](https://semver.org/#spec-item-11) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.1.1 |
| **Related**  | [D-0004](#d-0004), [D-0016](#d-0016), [D-0024](#d-0024), [D-0028](#d-0028) |

> **Amended 2026-07-02 by [D-0070](#d-0070)**: the `SdmxVersion` clause below is revised. `VersionType`'s grammar is canonical (one lexeme per value), so the stored raw was a redundant copy of the decomposition and is dropped: `SdmxVersion` is now the statedness-preserving `{ major, minor: Option<u32>, patch, extension }` with structural equality and a reconstructing `Display`. The lossless-raw rule below narrows to non-canonical grammars; `SdmxDecimal`/`SdmxInteger` stand unchanged. Original body retained for provenance.
>
> **Amended 2026-07-02 by [D-0073](#d-0073)**: consequence (3)'s follow-up question, whether the reference structs adopt `SdmxVersion`, is resolved with a different answer than it framed: they adopt `VersionRef`, the reference grammar, because a declaration version cannot carry the `+` wildcard forms every reference class admits.

**Observation**: Several SDMX fields are constrained lexical types — `xs:decimal`, `xs:integer`, `VersionType`, `StandardTimePeriodType` — whose value space does not map losslessly onto any fixed Rust type (`xs:decimal`/`xs:integer` are unbounded; version/time are structured grammars). The earlier "store the string, defer grammar to the parser" stance (D-0024 Tier-A; D-0016 for BCP-47) sits in tension with D-0004/D-0019 (construction enforces invariants for *all* callers, not just the parser) and make-illegal-states-unrepresentable.

**Decision**: Model each as a **validated newtype with lossless `String` storage**: `raw` holds the canonical lexical form verbatim (never normalised); `new()` validates the grammar at construction (cheap, hand-rolled, `no_std`, no regex crate) — *not* deferred; where validation naturally classifies the value, retain that as cheap derived fields alongside `raw`. Concrete types:

- **`SdmxDecimal(String)`** / **`SdmxInteger(String)`** — bare validated newtypes (no useful sub-kind). Distinct types so a *coded* facet cannot hold a fractional value (`"2.5"` unrepresentable in `SdmxInteger`). `From<SdmxInteger> for SdmxDecimal` is total/infallible/zero-cost (every integer lexeme is a decimal lexeme — "integers ⊂ decimals" made executable). `TryFrom<SdmxDecimal> for SdmxInteger` is a strict **lexical** validation, not a conversion: succeeds iff the string is *already* a valid `xs:integer` lexeme, never rewrites it — `"42"`→Ok, `"2.5"`→Err, **`"42.0"`→Err** (fractional syntax; rejected, not normalised, to keep `raw` lossless).
- **`SdmxVersion { raw, major: u32, minor: u32, patch: Option<u32>, extension: Option<String> }`** — full `VersionType` grammar validated; `patch:None` encodes the legacy form (so semantic-vs-legacy is structural, no separate kind enum); `extension` is the semantic prerelease suffix (lossless for `1.0.0-rc` etc.). `PartialEq`/`Eq` compare `raw`; `Ord`/`PartialOrd` **adhere to semver §11 precedence** (numeric triple, then prerelease rules; `1.0.0-rc < 1.0.0`) — transcribed, not invented.
- **`SdmxTimePeriod { raw, kind: SdmxTimePeriodKind }`** — `kind` mirrors `StandardTimePeriodType` 1:1 (4 Gregorian/DateTime + 7 Reporting = 11 exhaustive variants). A `granularity()` accessor projects onto a calendar-system-agnostic `Granularity` (Year/.../Instant) — the plain-name view without duplicating variants.

`Error` gains `InvalidDecimal`/`InvalidInteger`/`InvalidVersion`/`InvalidTimePeriod`.

**Rationale**: `String` is the *only* lossless rep of unbounded `xs:decimal` (`f64` rounds; `rust_decimal` is bounded — and would add a dependency to the foundational crate). Validating at `new()` rather than deferring honours the single-write-path contract (D-0004) for hand-built objects too. Retaining the discriminant is free (validation already traverses the string) and gives consumers the branch they want (version comparison, period granularity) without re-parsing.

**Consequences**: (1) **Naming rule generalised** — an SDMX lexical newtype carries the `Sdmx` prefix *when its bare name collides with a well-known external type in normal use* (`Decimal`↔`rust_decimal`, `Integer`↔primitives, `Version`↔semver, `TimePeriod`↔`chrono`). Distinctive domain names (`Codelist`, `Dimension`, `CubeRegion`, …) and the per-crate `Error` (ADR-0006: path-disambiguated `sdmx_types::Error`, Rust convention) stand bare. (2) **D-0024 revised** — its Tier-A grammar deferral withdrawn; `Version`→`SdmxVersion`. (3) The reference structs' `version: String` (D-0024 consequence 4) remains a separate follow-up — whether they adopt `SdmxVersion`. (4) No new dependency — all validators hand-rolled.

---

### D-0028 — Component Representation subsystem

| **Area**     | Data structure |
| **Phase**    | Phase-1 |
| **Status**   | Active (subsystem amended by [D-0048](#d-0048), [D-0052](#d-0052); facet-grammar cut closed by [D-0076](#d-0076)) |
| **Keywords** | representation, textformat, datatype, facets, codelist, component, spec-alignment |
| **Spec ref** | [SDMXStructureBase.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureBase.xsd#L209-L242) (`RepresentationType`, the TextFormat tier chain, `CodeDataType`); [SDMXStructureDataStructure.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureDataStructure.xsd#L609-L625) (`SimpleDataStructureRepresentationType`); [SDMXStructureConcept.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureConcept.xsd#L101-L120) (`ConceptRepresentation`); [SDMXCommon.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXCommon.xsd#L1244-L1470) (`DataType` subsets); 3.0 identical throughout except the `isMultiLingual` default ([D-0046](#d-0046)) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.6.1 |
| **Related**  | [D-0021](#d-0021), [D-0025](#d-0025), [D-0027](#d-0027), [D-0029](#d-0029), [D-0048](#d-0048), [D-0052](#d-0052), [D-0076](#d-0076) |

> **`xs:duration` facet-grammar cut closed 2026-07-08 by [D-0076](#d-0076).** `TextFormat.time_interval` and `EnumerationFormat.time_interval` now carry the `SdmxDuration` lexical newtype, so consequence (4)'s cut list (`ConceptRole`, `ISOConceptReference`, the `xs:duration` facet grammar) is fully closed.
>
> **`ISOConceptReference` cut closed 2026-07-07.** `Concept` now carries `iso_concept_reference: Option<IsoConceptReference>`, a pub-field carrier of the element's three mandatory strings. The `xs:duration` facet grammar is the sole remaining cut of consequence (4).
>
> **`ConceptRole` cut closed 2026-07-07; site list corrected.** `Dimension`, `Attribute`, and `Measure` now carry `concept_roles: Vec<ConceptReference>` (0..* in wire order, so a possibly-empty `Vec`, not an `Option`). Consequence (4) mis-states the sites as "dimensions/measures/concepts": the schema declares `ConceptRole` on dimensions, attributes, and measures only, and `TimeDimensionType`'s restriction drops it, so `TimeDimension` correctly omits it. `ISOConceptReference` and the `xs:duration` facet grammar remain cut.
>
> **Further amended by [D-0052](#d-0052)**: `text_type` is stored as `Option<DataType>` — the `String` default is an effective view (position-aware: the time tier defaults to `ObservationalTimePeriod`).
>
> **AMENDED 2026-06-11 by [D-0048](#d-0048)** Four corrections/completions to the subsystem below: (1) the enumeration reference is **not** codelist-only — the concept/attribute/measure positions admit `Codelist` *or* `ValueList` (`AnyCodelistReferenceType`); (2) `pattern` is optional on the **uncoded** `TextFormatType` too — the "present on coded; absent is the trade" claim below is wrong against both XSDs; (3) `TextFormatType` also carries `isMultiLingual` (with a 3.0/3.1-flipped default — D-0046), unmodelled below; (4) the representation-level `minOccurs`/`maxOccurs` promised "on the component wrapper" were never drawn, and `maxOccurs` is `OccurenceType` (a number **or** `"unbounded"`), which `Option<u32>` cannot hold. D-0048 re-draws the subsystem (wrapper-struct `Representation`, widened `EnumerationReference`, full facet set) and enforces the per-position mechanical restrictions at the component constructors. The coded/uncoded split, facet typing, and 44-value `DataType` below all stand.

**Observation**: A component's representation (`LocalRepresentation`; a concept's `CoreRepresentation`) declares how it is typed/valued. `RepresentationType` is a **choice**: `Enumeration` (codelist ref, the coded case) + optional `EnumerationFormat`, OR `TextFormat` (uncoded facet bundle), plus representation-level `minOccurs`/`maxOccurs`. The design previously modelled only the coded case, ad-hoc, as `codelist: Option<CodelistReference>` — a half-modelled representation that dropped all text-format facets (`textType` and 13 others) and the concept's core representation.

**Decision**: Model the full subsystem. `Representation = Enumeration { codelist, format: Option<EnumerationFormat> } | TextFormat(TextFormat)`. `DataType` is the `textType` facet — **all 44 values, exhaustive** (D-0021; identical 3.0/3.1). `TextFormat` carries the 14 uncoded facets (numerics = `SdmxDecimal`, time facets = `SdmxTimePeriod`, lengths/`decimals` = `u32`); `EnumerationFormat` (coded, `CodedTextFormatType`) carries the near-subset with **integer** numerics (`SdmxInteger` — coded values are discrete, so `"2.5"` is unrepresentable), no `decimals`, plus `pattern`. The ad-hoc `codelist` field on `Dimension`/`Attribute`/`Measure` is **replaced** by `representation: Option<Representation>` (`Option` per `LocalRepresentation` `minOccurs="0"`; mandatory on `TimeDimension`).

**Rationale**: The coded/uncoded split, the integer-vs-decimal facet typing, and the 44-value `DataType` are all the spec's — a benchmark canonical model carries them rather than narrowing to codelist-only. Two facet types (not one) preserve the discreteness constraint of coded representations (D-0027's `SdmxInteger`/`SdmxDecimal` split).

**Consequences**: (1) Components gain custom `Deserialize` via their existing validated `new()` (already custom for the NCName check — D-0025). (2) Representation-level `min_occurs`/`max_occurs` are carried on the component wrapper, not the `Representation` enum. (3) **Closes the deferred representation gaps** — `Concept` gains `core_representation: Option<Representation>` (closing the `CoreRepresentation` half of review M4), and `Measure`/`Dimension`/`Attribute`/`TimeDimension` carry `representation` (closing the D-0025 `LocalRepresentation` cut). (4) **Still cut** (small, flagged): `ConceptRole` (a concept-role reference list on dimensions/measures/concepts), `ISOConceptReference` (ISO 11179 ref on `Concept`), and `xs:duration` facet *grammar* (`time_interval` kept as a lexical `String` — an uncommon facet, unlike the core numerics).

---

### D-0029 — TimeDimension modelled as a separate Option slot on the DSD

| **Area**     | Data structure |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | time-dimension, dsd, dimension-list, position, representation, spec-alignment |
| **Spec ref** | [SDMXStructureDataStructure.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureDataStructure.xsd#L314-L326) (`DimensionListType`, `TimeDimensionType`, `TimeDimensionRepresentationType`) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §4, §5.6 |
| **Related**  | [D-0022](#d-0022), [D-0025](#d-0025), [D-0028](#d-0028) |

**Observation**: `DimensionListType` is `Dimension+ , TimeDimension?`. `TimeDimension` is a *distinct* element (`TimeDimensionType`): id `fixed="TIME_PERIOD"` (NCName), `position` **prohibited** (it is not part of the ordered key), and a **mandatory** `LocalRepresentation` restricted to a time `TextFormat` (no enumeration). The design's flat `Vec<Dimension>` erased it entirely.

**Decision**: Add `time_dimension: Option<TimeDimension>` to `DataStructureDefinition`, separate from the ordered `dimensions: Vec<Dimension>`. `TimeDimension { metadata, concept, representation: Representation }` — no `position` (prohibited), and `representation` is mandatory (not `Option`), constrained to the `TextFormat` arm (D-0028). NCName-validated `new()` + custom `Deserialize`, like the other components (D-0025).

**Rationale**: Mirrors the spec's own `Dimension+ , TimeDimension?` structure: the time dimension is genuinely not a member of the ordered key (no position), so folding it into the `Vec` would misrepresent it. A separate `Option` captures "this DSD has a time dimension" distinctly.

**Consequences**: (1) `DataStructureDefinition` gains the field; its position-canonicalisation invariant (D-0022) is unaffected (the time dimension has no position to canonicalise).

---

### D-0030 — External-reference modelled on MaintainableMetadata (Infoset Store + derived view)

| **Area**     | Maintainable artefacts |
| **Phase**    | Phase-1 |
| **Status**   | Active (mechanism amended by [D-0031](#d-0031), [D-0052](#d-0052)) |
| **Keywords** | maintainable, external-reference, service-url, structure-url, superset, wire-store, round-trip, spec-alignment |
| **Spec ref** | [SDMXCommon.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXCommon.xsd#L394-L418) (`MaintainableType`, `ExternalReferenceAttributeGroup`); [3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXCommon.xsd#L390-L409) (identical) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.2 |
| **Related**  | [D-0010](#d-0010), [D-0014](#d-0014), [D-0016](#d-0016), [D-0019](#d-0019), [D-0023](#d-0023), [D-0031](#d-0031) |

> **Amended 2026-06-11 by [D-0052](#d-0052)**: `is_external_reference` is stored as `Option<bool>` (stated-vs-absent preserved; `false` is the effective view's default).
>
> **Amended 2026-06-10 by [D-0031](#d-0031)** (mechanism re-homed under the two-layer model). The *finding* stands (the three attributes were dropped and must be modelled); the *shape* changed. The original decision (below) used an `ExternalReference = Local | External` enum whose variant tag was `isExternalReference`, with `MaintainableMetadata::new()` **rejecting** the schema-valid `isExternalReference=false` + URL-present combination as "incoherent." D-0031 (two-layer: Infoset Store + derived views; `new()` rejects only schema-*invalid* input) forbids that rejection — the combination is schema-valid, so it must round-trip, not error. **Revised shape:** store the three attributes verbatim — `is_external_reference: bool` (default false) and `service_url: Option<String>` / `structure_url: Option<String>` (both `xs:anyURI`, unvalidated per D-0014) — as plain fields on `MaintainableMetadata`; every schema-valid combination is representable and round-trips. The "URLs imply external" coherence is exposed as a **derived view / lint** (`is_external_reference()` reads the stored bool; a coherence lint flags `false`+URL), never enforced at construction. `Link` stays omitted pending the M6 re-examination of D-0014 (see resume memory). The original enum body is retained below for provenance.

**Observation**: Every `MaintainableType` carries three attributes the design dropped: `isExternalReference` (`xs:boolean`, default `false`), and — via `ExternalReferenceAttributeGroup` — `serviceURL` and `structureURL` (both `xs:anyURI`, optional). Identical in 3.0 and 3.1. (The attribute's prose mentions `registryURL`/`repositoryURL`, but those are **not declared attributes**; only `serviceURL` and `structureURL` exist. The same attribute group also appears on `PayloadStructureType` — a message-header concern, out of domain-type scope.) An external-reference stub (id+urn only, body resolved elsewhere) is a real wire shape the model could not flag.

**Decision (superseded shape — retained for provenance; see the amendment note above)**: Add `external_reference: ExternalReference` to `MaintainableMetadata`, where `ExternalReference = Local | External { service_url: Option<String>, structure_url: Option<String> }`. The variant tag *is* `isExternalReference`: `Local` ⟺ `false`, `External { .. }` ⟺ `true`. URLs are `Option<String>` (unvalidated `xs:anyURI`, per D-0014); `Link` stays omitted (D-0014). `MaintainableArtefact` gains `fn external_reference(&self) -> &ExternalReference`, with `is_external_reference()` derived from the variant. `MaintainableMetadata::new()` (already fallible, D-0023) **validates** the coherence the schema's type system cannot express: a `serviceURL`/`structureURL` present together with `isExternalReference=false` is rejected (`Err`). `External { None, None }` (external ref, URLs to be resolved by other means) is valid and representable.

**Rationale (of the superseded shape)**: Encodes the spec's *intended* invariant directly in the type — the URLs are defined by the attribute's own documentation as the resolution targets **of an external reference**, so they are meaningful only when `isExternalReference=true`. *(Superseded:)* this reasoning treated `isExternalReference=false` + URL-present as "incoherent, reject it"; D-0031 reframes that as a strict violation of ADR-0023 — destroying a schema-valid wire shape at the store for convenience. The coherence intuition is correct but belongs in a **view/lint**, not in `new()`.

**Consequences**: (1) `MaintainableMetadata` gains the three verbatim fields; `new()` adds **no** new rejection for them (it stays fallible only for the D-0023 agencyID check). (2) `is_external_reference()`/URL accessors are derived views (D-0031); the `false`+URL coherence concern is lint-only. (3) First worked example of D-0031.

---

### D-0031 — Two-layer model: Infoset Store + derived views (never collapse the store for convenience)

> **Promoted to [ADR-0023](adr/0023-two-layer-infoset-store-and-derived-views-architecture.md)** (2026-06-11) as the foundational architecture commitment, in its matured form: Layer 1 is *Infoset-complete within schema content* — including element order and duplicates ([D-0051](#d-0051)) and attribute statedness, since **XSD defaulting is a view over the data, not the data itself** ([D-0052](#d-0052)). The ADR is now the authoritative statement; this entry is retained as the audit trail of the rule's discovery.
>
> **Reject-line amended 2026-06-11 by [D-0059](#d-0059)** (in ADR-0023, the authoritative statement): mechanical schema invalidity is the *ceiling* of `new()` rejection, not a mandate — structural and identity/grammar-bearing invalidity stays rejected, but a value-level lexeme in a content slot the store can hold verbatim may be ruled stored-plus-linted (the parsable-within-spec principle; first instance: the `LocalisedString` language key). Existing rejection sites are unchanged unless individually re-ruled.

| **Area**     | Architecture (foundational) |
| **Phase**    | Phase-1 |
| **Status**   | Promoted(ADR-0023) (reject-line amended by [D-0059](#d-0059)) |
| **Keywords** | round-trip, wire-store, derived-view, lint, canonical-superset, foundational, supersedes |
| **Spec ref** | [ADR-0008](adr/0008-model-sdmx-3-0-and-3-1-divergence-with-a-unified-constraintmodel.md) (guardrail #1, lossless canonical superset); all pinned XSDs (`specs/3.0`, `specs/3.1`) as ground truth |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §3, §5.2, §7 |
| **Related**  | [D-0004](#d-0004), [D-0016](#d-0016), [D-0017](#d-0017), [D-0022](#d-0022), [D-0024](#d-0024), [D-0027](#d-0027), [D-0030](#d-0030) |

**Observation**: Across several decisions the model traded strict spec adherence for consumer ergonomics by **collapsing a wire distinction in storage** — `Dimension.position` canonicalised to a mandatory `u32` (D-0022, erasing implicit-vs-explicit); blank/whitespace `xml:lang`-keyed *values* rejected at construction (D-0016, though `TextType` = bare `xs:string` permits them); the `isExternalReference`+URL triple about to be rejected when "incoherent" (D-0030 original). Each is locally justifiable but shares one shape: a schema-*valid* wire input is **made unrepresentable or rejected** so the in-memory form is tidier. For a crate whose reason to exist is the lossless canonical superset (ADR-0008 #1), that is a critical design flaw — it is the same defect class as the valid `Code` ids rejected and the `Link` valid content dropped: the model disagreeing with the spec about what is representable.

**Decision**: Adopt a **two-layer model**. *Layer 1 — the store* is a verbatim representation of the wire: it preserves every distinction a schema-valid document can express, and `new()`/`Deserialize` reject **only** schema-*invalid* input (off-pattern ids, missing required elements, malformed lexemes — i.e. exactly what the XSD itself rejects). *Layer 2 — views* sit on top: derived accessors, canonical projections, and non-destructive **lints** provide the convenience that earlier decisions sought, computed from the Infoset Store, never baked into it. The governing rule, stated once:

**What `new()` may reject — the mechanical-vs-prose line.** "Schema-invalid" means **mechanically** schema-invalid: what an XSD validator (`xmllint --schema`) would reject — pattern/type facets, `minOccurs`/`maxOccurs` cardinality, enumerations, required attributes/elements. Constraints the spec expresses **only in prose** (an `<xs:documentation>` annotation, not a machine-checkable facet) are **not** grounds for `new()` rejection: a document violating a prose rule still *validates* against the XSD, so it is schema-valid input the store must hold. Such rules become **lints** (Layer 2). Worked example: `Dimension.position` "*if specified must be consistent with* the key-descriptor position" is prose — a DSD with a contradictory stated position passes XSD validation — so the consistency check is a lint, not a `DSD::new()` rejection (this is what overturned D-0022's sort/canonicalise; see D-0022's supersession note). The test is mechanical, leaving no subjectivity: *if a validator wouldn't reject it, neither do we.*

> **Friction is resolved by adding a view, never by collapsing the store. When something is inconvenient to consume raw, derive a view. Never pay for convenience by destroying wire information in the store.**

**Rationale**: (1) **Makes round-trip exactness structural, not a judgment call.** "Is this safe to collapse?" was the flaw that let the `Link` mischaracterisation and the M2 "incoherent, reject it" framing survive — each was a *flawed justification* to drop schema-valid content. Removing collapse as an available move closes the architectural flaw mechanically: the store never drops anything, so there is nothing to mis-justify. (2) **Loses none of the convenience.** D-0022's canonical `position`, D-0024's canonical version, D-0030's `is_external_reference()` — all survive as *views*; only their storage moves from collapsed to verbatim. (3) **Coherence/quality concerns get a non-destructive home** (lints over the store) instead of `new()` rejections, so the model can *hold and flag* a spec-permitted-but-dubious shape rather than refusing it. (4) Aligns the whole crate to one testable contract: *if the spec can express it, the store can round-trip it.*

**Consequences** (updated systematically across the codebase):
1. **Superseded the "collapse-in-store" mechanism of several decisions; re-homed their canonical conclusions to views.** Their *findings* stand; their *storage* went verbatim. Swept: **D-0022** (`position` → verbatim `Option<i32>` store; `Dimension::effective_position(index)` view; consistency-with-order is a prose constraint → lint, not a rejection; `DSD::new()` no longer sorts/canonicalises — see D-0022 supersession note), **D-0016** (its *value* rejection withdrawn → blank values stored + lint; its *key* rejection **stays**, because the key is `xs:language` and a blank key is mechanically schema-*invalid* — mechanically exact, not an invented constraint; a factual error in its Observation corrected — see D-0016 amendment note), **D-0024** (audited: already compliant — `Option<SdmxVersion>` + verbatim `raw`, defaulting-collapse was already rejected; confirmed, no change), **D-0017** residue (already withdrawn by D-0026). **D-0019**, **D-0023**, **D-0027** unaffected — they reject only schema-*invalid* input, which Layer 1 still does.
2. **D-0030 (M2) re-homed** as the first worked example: verbatim `is_external_reference`+URL fields, coherence as view/lint. See D-0030's amendment note.
3. **Trait/accessor convention (settled in the sweep)**: inheritance-trait accessors are Layer-2 and may return derived/canonical values (`is_external_reference()`, `version_display()`); a `Dimension` exposes both `position()` (Layer-1, raw `Option<i32>`) and `effective_position(index)` (Layer-2, derived). The rule: where a field's raw and canonical forms differ, expose **both** — never let a trait hide wire state behind a lossy accessor.
4. **Coherence-lint surface (catalogued, build deferred).** D-0031 names a non-destructive lint layer but commits **no** Phase-1 lint implementation — consistent with the verification-before-mutation discipline. The catalogue has a single citable home: the design doc's "Catalogued Lints (Layer 2, not built)" subsection ([§5.11](design/0010-sdmx-core-domain-types-design.md)), each member citing its source D-number (e.g. blank localised-string *value*; `Dimension` stated-`position` vs list order; `isExternalReference=false` with a URL present). The design doc is the home rather than an ADR-0023 appendix or a standalone doc because it is the living blueprint implementers must read; the lint *subsystem* design (predicates as code, reporting shape) is future work that will cite §5.11 as its requirements list. **Standing rule: a new lint is added to §5.11 in the same change as the decision that names it.** These are *where lints will live when built*, not committed Phase-1 deliverables.
5. **Cost accepted**: Layer 1 carries more `Option`s/raw forms than the collapsed design did; the §5.2 metadata structs carry a store/view split. The price of the guarantee, paid once structurally rather than re-litigated per field.

---

### D-0032 — ItemScheme.isPartial modelled on ItemScheme, distinct from isPartialLanguage

> **Amended 2026-06-11 by [D-0052](#d-0052)**: `is_partial` is stored as `Option<bool>` — the "no Option/collapse question arises" reasoning below treated the schema default as data; XSD defaulting is a view. The placement decision (on `ItemScheme`, not `MaintainableMetadata`) stands.

| **Area**     | Item schemes |
| **Phase**    | Phase-1 |
| **Status**   | Active (statedness storage amended by [D-0052](#d-0052)) |
| **Keywords** | item-scheme, is-partial, partial-codelist, constraint-context, superset, spec-alignment |
| **Spec ref** | [SDMXStructureBase.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureBase.xsd#L17-L35) (`ItemSchemeType.isPartial`); [3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXStructureBase.xsd#L11-L29) (identical) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.5 |
| **Related**  | [D-0010](#d-0010), [D-0013](#d-0013), [D-0031](#d-0031) |

**Observation**: `ItemSchemeType` (the abstract base for `Codelist`/`ConceptScheme`/`AgencyScheme`) adds `isPartial` (`xs:boolean`, optional, default `false`) on top of `MaintainableType` — identical in 3.0 and 3.1. Per its documentation it indicates "only the relevant portion of the item scheme is being communicated … when a codelist is returned for a data structure in the context of a constraint." Since this crate models constraints (D-0013), the partial-scheme flag is in-scope; the design did not model it. It is **distinct** from `MaintainableType.isPartialLanguage` (D-0010): `isPartial` = an incomplete set of **items**; `isPartialLanguage` = an incomplete set of **languages**.

**Decision**: Add `is_partial: bool` to `ItemScheme<I>` (default `false`), with `ItemScheme::new(metadata, is_partial)` and an `is_partial()` accessor. Concrete wrappers (`Codelist`, `ConceptScheme`, `AgencyScheme`) forward it via their own `is_partial()`. It lives on **`ItemScheme`, not `MaintainableMetadata`** — the spec attaches `isPartial` to the item scheme, not the maintainable base, so DSD/`Dataflow`/`DataConstraint` ([D-0037](#d-0037)) — maintainable, non-scheme — do not carry it; modelling it on the shared metadata would over-reach the superset (a field on types that cannot express it). Not exposed on the `MaintainableArtefact` trait for the same reason — there is no item-scheme trait (it is a concrete generic), so a plain accessor is correct.

**Rationale**: Mirrors the spec's inheritance precisely — same reasoning that keeps `measures` on the DSD (D-0025) and not on shared metadata. A plain stored `bool` suffices: `isPartial` has a schema default of `false`, so absent→`false` is the spec's own canonicalisation and round-trips identically — no `Option`/collapse question arises under D-0031 (the field is verbatim as a bare `bool`).

**Consequences**: (1) `ItemScheme<I>::new()` gains the `is_partial` parameter; the three wrapper `new()`s thread it through. (2) `AgencyScheme` stays a derived transparent carrier — `is_partial` is just another invariant-free field in the inner scheme.

---

### D-0033 — Annotations modelled on every AnnotableType descendant (universal extension point)

| **Area**     | Annotation |
| **Phase**    | Phase-1 |
| **Status**   | Active (forward rule amended by [D-0049](#d-0049)) |
| **Keywords** | annotation, annotable, extension-point, universal, superset, placement-principle, spec-alignment |
| **Spec ref** | [SDMXCommon.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXCommon.xsd#L195-L202) (`AnnotableType`, `AnnotationType`, `AnnotationsType`); [SDMXStructureConstraint.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureConstraint.xsd#L240-L262) (`AvailabilityConstraintType`, `RegionType`); 3.0 equivalents |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §4, §5.8 |
| **Related**  | [D-0001](#d-0001), [D-0011](#d-0011), [D-0013](#d-0013), [D-0014](#d-0014), [D-0031](#d-0031) |

> **Forward rule AMENDED 2026-06-11 by [D-0049](#d-0049).** The audit below counted only types the model *had*; it missed a third category: **identifiable annotable wire structures the model had flattened away** — the DSD component descriptors (`DimensionList`/`Group`/`AttributeList`/`MeasureList`), which are `IdentifiableType` descendants whose annotations/links had no home in the flattened collections. D-0049 models the descriptors, restoring those chokepoints. The forward rule is restated: a new annotation-home gap can arise when (a) a **non-identifiable annotable** type is modelled (bare field — the rule below; D-0039's `DataKey` and D-0047's `ValueItem` followed it), **or** (b) an annotable wire structure is **flattened** rather than modelled — flattening therefore requires an explicit recorded cut naming the annotation loss, never a silent collapse.

**Observation**: SDMX `AnnotationType` is the spec's **universal, producer-defined extension point** — its own documentation: "non-documentation notes and annotations … various uses … **not enumerated, as these can be specified by the user or creator** of the annotations." `AnnotableType` sits at the root of the type hierarchy (`IdentifiableType` extends it, and so do several non-identifiable types) precisely because annotations can appear on **any** artefact, unpredictably by design. The design modelled annotations only where they arrive via `IdentifiableMetadata`; a full `AnnotableType`-descent audit (3.0 + 3.1) found **two in-scope annotable types with no annotation field**: `AvailabilityConstraint` (`AvailabilityConstraintType` → `AnnotableType` directly; 3.1-only) and `CubeRegion` (`CubeRegionType` → `RegionType` → `AnnotableType`; both versions). The region-internal value-sets (`MemberSelectionType` → `KeyValueSelection`/`ComponentSelection`; since [D-0038](#d-0038) the node types are `CubeRegionKey`/`ComponentValueSet`) are **bare** — NOT annotable — in both versions, so annotability stops at the region boundary.

**Decision**: State the placement principle: **the canonical superset models `annotations` on every domain type that maps to an `AnnotableType` descendant** — because annotations are a universal producer-defined channel whose placement is unpredictable by design; rarity-on-a-given-type is exactly the assumption the universal-annotation design exists to defeat, so it is not grounds to omit. Mechanism: **via `IdentifiableMetadata.annotations` if the type is identifiable** (the single chokepoint covering all codes/concepts/agencies/schemes/components/maintainables — they reach `AnnotableType` *through* `IdentifiableType`), **else a bare `annotations: Vec<Annotation>` field** on the type. The only types needing a bare field are the **non-identifiable annotable** ones; the audit found exactly two, now added: `AvailabilityConstraint`, `CubeRegion`. Both stay invariant-free pub-field carriers (derived `Deserialize`).

**Rationale**: (1) **Not consistency-for-its-own-sake** — the hierarchy is universal *because the data is*, so modelling annotations universally implements the spec's extension mechanism verbatim rather than transcribing its class graph. Dropping them on `AvailabilityConstraint`/`CubeRegion` would silently discard producer-defined metadata exactly where a producer chose to attach it — a lossless-superset defect (ADR-0008 #1), same class as the `Link` drops. (2) **`Vec` with empty ≡ absent, not `Option<Vec>`** — `Annotations` is `minOccurs="0"` (absent = none) and `AnnotationsType` mandates ≥1 `Annotation` (present-but-empty is schema-invalid), so the wire has exactly two states mapping 1:1 to empty/non-empty `Vec`; `Option<Vec>` would manufacture an unrepresentable `Some(empty)` state and is rejected. Annotations are purely additive notes, so there is no meaningful "deliberately zero" state to preserve (contrast D-0026's CubeRegion `Empty` value-set, which *was* meaningful). Verbatim under D-0031: the writer emits `<Annotations>` iff non-empty. (3) **Self-checking forward rule** — `IdentifiableMetadata` is the single chokepoint for identifiable annotability, so a *new* bare-field gap can only arise when a **non-identifiable annotable** type is modelled (a type with no id that still takes notes — the conspicuous constraint-region shape). New annotable types added later (other crates / later phases — data-message, metadata-message, process, mapping types are all `AnnotableType` descendants but out of 0010's scope) inherit this rule without re-auditing.

**Consequences**: (1) `CubeRegion` and `AvailabilityConstraint` each gain `annotations: Vec<Annotation>`; both remain pub-field derived carriers (annotations carry no invariant). (2) D-0013 (AvailabilityConstraint has no `MaintainableMetadata`) is unchanged and clarified: non-maintainable ≠ non-annotable — the constraint asymmetry is about *maintainability*, not annotability. (3) The two-field set is complete for everything 0010 models; coverage of future types is guaranteed by the principle, not by re-audit. (4) 3.0/3.1 divergence noted: `AvailabilityConstraint` is 3.1-only; `CubeRegion` annotability is in both — the superset carries both regardless.

---

### D-0034 — ConstraintAttachment split into two exhaustive per-constraint enums (XSD restriction encoded)

| **Area**     | Constraints |
| **Phase**    | Phase-1 |
| **Status**   | Active (target enumeration amended by [D-0044](#d-0044)) |
| **Keywords** | constraint, attachment, restriction, exhaustive, non-empty-vec, cardinality, reference-types, spec-alignment |
| **Spec ref** | [SDMXStructureConstraint.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureConstraint.xsd#L146-L192) (`ConstraintAttachmentType`, `DataConstraintAttachmentType`, `AvailabilityConstraintAttachmentType`, `MetadataConstraintAttachmentType`); [SDMXCommonReferences.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXCommonReferences.xsd#L952-L959) (`DataProviderReferenceType` = OrganisationReferenceType, `ProvisionAgreementReferenceType`) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §4, §5.8 |
| **Related**  | [D-0002](#d-0002), [D-0013](#d-0013), [D-0019](#d-0019), [D-0021](#d-0021), [D-0031](#d-0031), [D-0033](#d-0033) |

> **AMENDED 2026-06-11 by [D-0044](#d-0044).** The "4 targets" enumeration below is accurate for **3.1 only**: 3.0's `DataConstraintAttachmentType` additionally allows a `SimpleDataSource` choice arm (`xs:anyURI`, 1..unbounded) and trailing `QueryableDataSource` elements (0..unbounded) on each of the three 1..\* reference sequences — both removed from constraint attachments in 3.1. D-0044 models them as 3.0-only superset members. The restriction-encoding decision below otherwise stands.

**Observation**: The `ConstraintAttachment` `#[non_exhaustive]` comment under-enumerated (listed 3, the abstract base has 8). Verification showed the issue is deeper: the spec has **three concrete restrictions** of the abstract `ConstraintAttachmentType`, and the model used one flat shared enum (a single `Dataflow(DataflowReference)` variant) for both constraint types. The restrictions: `DataConstraintAttachmentType` → DataConstraint = {DataProvider, DataStructure, Dataflow, ProvisionAgreement} (4); `AvailabilityConstraintAttachmentType` → AvailabilityConstraint = {DataStructure, Dataflow, ProvisionAgreement} (3, each `maxOccurs="1"`); `MetadataConstraintAttachmentType` → a metadata-constraint type **not modelled in 0010** = the 4 metadata targets. Two further facts: (a) the restriction is *mechanical* (XSD `restriction`), so an availability-attached-to-DataProvider is genuinely schema-invalid, not merely odd; (b) in the data constraint, DataStructure/Dataflow/ProvisionAgreement are `maxOccurs="unbounded"` (1..*), which the flat single-ref enum **could not represent** — a latent cardinality bug.

**Decision**: Replace the shared `ConstraintAttachment` with **two per-constraint enums mirroring the XSD restrictions**, both **exhaustive** (D-0021 — bounded, spec-fixed):
- `DataConstraintAttachment` = `DataProvider(DataProviderReference)` | `DataStructure(DataStructureRefs)` | `Dataflow(DataflowRefs)` | `ProvisionAgreement(ProvisionAgreementRefs)` — on `DataConstraint`.
- `AvailabilityConstraintAttachment` = `DataStructure(DataStructureReference)` | `Dataflow(DataflowReference)` | `ProvisionAgreement(ProvisionAgreementReference)` (all single) — on `AvailabilityConstraint`.

The three 1..* data arms wrap **bespoke non-empty-vec newtypes** (`DataStructureRefs`/`DataflowRefs`/`ProvisionAgreementRefs`) — *not* a generic `NonEmptyVec<T>`: the arms carry distinct domain identity (a vec of dataflow refs is not interchangeable with a vec of DSD refs), warrant distinct empty-error variants naming *what* was empty, and may gain arm-specific behaviour; same bespoke pattern as `DimensionRefs` (D-0019), private field + validating `new()` (empty = mechanically schema-invalid for a chosen unbounded `<choice>` arm, so `new()`-rejectable under D-0031) + custom `Deserialize`. `Error` gains `EmptyDataStructureRefs`/`EmptyDataflowRefs`/`EmptyProvisionAgreementRefs`. Two new reference structs added: `ProvisionAgreementReference` (maintainable URN → flat agency/id/version) and `DataProviderReference` (its spec type `OrganisationReferenceType` shares `ComponentUrnReferenceType` with `ConceptReferenceType`, so it takes the **item-in-scheme** shape agency/scheme_id/id, *not* the maintainable triple).

**Rationale**: Encoding the restriction in the type makes the illegal cross-attachment (e.g. availability-on-DataProvider) unrepresentable — mathematically exact to the spec's *mechanical* restriction, and the same architectural intent as the two `CubeRegion` selection maps (D-0026) and the `SdmxInteger`/`SdmxDecimal` split (D-0027). A flat shared enum would be over-permissive *and* (as built) lossy on the 1..* cardinality. The split also adds a second axis to the intentional Reporting-vs-Availability asymmetry formalised in D-0033 (maintainability + attachment subset). Bespoke-not-generic newtypes: incorrectly applied DRY — a generic container erases the per-arm domain identity and collapses three distinct empty-errors into one.

**Consequences**: (1) `ReportingConstraint.attachment` → `DataConstraintAttachment`; `AvailabilityConstraint.attachment` → `AvailabilityConstraintAttachment`; flat `ConstraintAttachment` removed. (2) Fixes the latent multi-dataflow cardinality bug (data constraint can now attach to multiple dataflows/DSDs/PAs). (3) **D-0021 now has no live `#[non_exhaustive]` instance** — `ConstraintAttachment` was its sole exemplar; the two replacements are exhaustive. The policy stands for future routine-growth enums; the exemplar is retired (D-0021 consequences updated). (4) **Metadata constraints + their 4 attachment targets remain out of 0010 scope** — a deliberate, recorded boundary (revisit if/when metadata constraints are modelled), not silent omission. (5) **Deferred to a D-0002 reference-types pass (inherited debt, none added):** all reference structs (old and new two) model references as parsed agency/id/version-style *fields*, where the XSD types are URN-pattern `simpleType`s; the URN-string-vs-parsed-fields question and `version: String`→`SdmxVersion` apply uniformly and are not resolved here.

---

### D-0035 — Link modelled on IdentifiableMetadata (reverses D-0014's omission)

| **Area**     | Identifiable artefacts |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | link, identifiable, hateoas, superset, supersedes-d0014, spec-alignment |
| **Spec ref** | [SDMXCommon.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXCommon.xsd#L278-L299) (`LinkType`, `IdentifiableType`); [3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXCommon.xsd#L274-L295) (identical) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.1, §5.2, §5.3 |
| **Related**  | [D-0014](#d-0014), [D-0031](#d-0031), [D-0033](#d-0033) |

**Observation**: D-0014 dropped `Link`, calling it "a transport-layer affordance belonging in the HTTP response envelope." Verification (surfaced during the M2 thread) shows that is a misreading: `LinkType` sits on `IdentifiableType` itself — `<xs:element ref="Link" minOccurs="0" maxOccurs="unbounded"/>`, identical in 3.0 and 3.1 — persisted in the structure message alongside `id`/`urn`/`uri`. It is a typed, multi-valued association: `rel` (required, `xs:string` — the relationship/type of object linked to), `url` (required, `xs:anyURI`), `urn` (optional registry URN of the target), `type` (optional `xs:string` media-type hint). It carries strictly **more** than the `uri` field D-0014 retained, and cannot be reconstructed from `uri`/`urn` (single identity fields, not a typed multi-valued relationship). Dropping it silently discarded producer-supplied domain content — a lossless-superset defect (ADR-0008 #1).

**Decision**: Model `Link { rel: String, url: String, urn: Option<String>, link_type: Option<String> }` and add `links: Vec<Link>` to `IdentifiableMetadata` — the single `IdentifiableType` chokepoint, so every identifiable artefact inherits it, exactly as `annotations` does (D-0033). Surface via a new `IdentifiableArtefact::links()` trait method (sibling of `annotations()`), delegated through the whole hierarchy and the concrete wrappers. `Vec<Link>` with empty ≡ absent (1:1 mapping with `minOccurs=0`/unbounded; plain `Vec`, not a non-empty newtype, because empty *is* schema-valid here — unlike the D-0034 choice arms). `rel` and `link_type` stay `String` (both bare `xs:string` with no enumeration — `type`'s "e.g. PDF, text, HTML" are examples; an enum would invent a constraint the wire does not impose, cf. `annotation_type` D-0011). `type` → `link_type` (Rust keyword). URLs unvalidated `xs:anyURI` (D-0014 `uri` precedent). `Link` is invariant-free: a pub-field derived carrier, riding `IdentifiableMetadata`'s existing custom `Deserialize` with no new validation.

**Rationale**: `Link` is genuine, persisted, multi-valued domain content the canonical superset must round-trip; "transport-layer" was the flawed justification (like M2's "incoherent" and the position "derivable") that D-0031 exists to foreclose. Placement on `IdentifiableMetadata` follows the same single-chokepoint logic the D-0033 annotation audit established (annotability/linkability both ride `IdentifiableType`).

**Consequences**: (1) **Supersedes the `Link`-omission clause of D-0014**; D-0014's `uri` addition stands unchanged. (2) `IdentifiableMetadata` gains `links` + ctor param; `IdentifiableArtefact` gains `links()` with delegation threaded through every impl (the same chain as `annotations()`). (3) The Drawbacks "`Link` Elements Omitted" entry is rewritten to record the reversal. (45) Identical in 3.0/3.1.

---

### D-0036 — DataConstraint cube regions capped at 2 (mechanical maxOccurs); pairing is a lint

| **Area**     | Constraints |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | constraint, cube-region, cardinality, maxOccurs, bounded-newtype, mechanical-vs-prose, lint, spec-alignment |
| **Spec ref** | [SDMXStructureConstraint.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureConstraint.xsd#L71-L91) (`DataConstraintType` → `CubeRegion minOccurs="0" maxOccurs="2"`); [3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXStructureConstraint.xsd#L75-L95) (identical) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §4, §5.8 |
| **Related**  | [D-0019](#d-0019), [D-0026](#d-0026), [D-0031](#d-0031), [D-0034](#d-0034) |

**Observation**: `DataConstraintType` declares `CubeRegion` as `minOccurs="0" maxOccurs="2"` (a **literal XSD facet**, verified raw, identical 3.0/3.1). The model used an unbounded `Vec<CubeRegion>` on `DataConstraint` — over-permissive against the wire. The element annotation ("a set of included or excluded regions can be described") implies one include + one exclude, but that pairing is **not** mechanically enforced and **not** explicitly stated: two `include="true"` regions pass XSD validation and are not contradicted by any prose (since a single region can only express the intersection of filters, multiple regions are required to express a union).

**Decision**: Cap via a bounded `CubeRegions(Vec<CubeRegion>)` newtype — private field, `new()` rejects `len() > 2` (→ `Error::TooManyCubeRegions`), custom `Deserialize` routing through `new()`. `DataConstraint.regions: Vec<CubeRegion>` → `CubeRegions`. **Empty is valid** (`minOccurs="0"` — a data constraint may be expressed purely via `DataKeySet`s), so unlike the D-0034 ref newtypes only the *upper* bound is enforced. The include/exclude pairing is **not** checked at `new()` (it is neither mechanical nor explicitly stated — encoding it would reject schema-valid wire); at most it is a non-destructive **lint** (catalogued, not built — D-0031).

**Rationale**: Textbook application of the D-0031 mechanical-vs-prose line: `maxOccurs="2"` is a mechanical facet an XSD validator enforces → `new()`-rejectable and structurally exact to encode; the "one include + one exclude" semantics are an inference of intent the schema declines to state → not a construction rejection (rejecting two-includes would be the blanket-NCName error class — stricter than spec on an inference). Bespoke newtype (not generic), per the D-0034 rationale: distinct domain identity + a named error. Mirror image of D-0034's ref newtypes — there a chosen `<choice>` arm made empty schema-invalid (reject empty); here `minOccurs="0"` makes empty schema-valid (allow empty, cap the top).

**Consequences**: (1) `DataConstraint.regions` becomes `CubeRegions`; `Error::TooManyCubeRegions` added. (2) `AvailabilityConstraint.region` is unaffected and confirmed correct — `AvailabilityConstraintType` declares its `CubeRegion` as `minOccurs="1" maxOccurs="1"`, matching the existing single non-optional `region: CubeRegion`. (3) The same-direction-regions case is a future lint, not a rejection — added to the D-0031 coherence-lint surface. (4) Out-of-scope parallel recorded: `MetadataConstraintType.MetadataTargetRegion` is also `maxOccurs="2"`, but the metadata-constraint type is not modelled in 0010 (same boundary as D-0034's metadata attachments).

---

### D-0037 — DataConstraint carries the 3.0 role as a verbatim superset member; ReportingConstraint renamed DataConstraint

| **Area**     | Constraints |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | constraint, role, allowed, actual, superset, divergence, naming, spec-alignment |
| **Spec ref** | [SDMXStructureConstraint.xsd 3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXStructureConstraint.xsd#L29-L54) (`ConstraintType.role` `use="required"`, `ConstraintRoleType`, `MetadataConstraintBaseType` `role` `fixed="Allowed"`); [3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureConstraint.xsd#L240-L262) (`role` zero occurrences; `AvailabilityConstraintType`) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5, §5.8; [ADR-0008](adr/0008-model-sdmx-3-0-and-3-1-divergence-with-a-unified-constraintmodel.md) |
| **Related**  | [D-0002](#d-0002), [D-0010](#d-0010), [D-0013](#d-0013), [D-0021](#d-0021), [D-0031](#d-0031), [D-0034](#d-0034), [D-0036](#d-0036) |

**Observation**: In 3.0, the abstract `ConstraintType` declares a `role` attribute (`ConstraintRoleType` = `Allowed` | `Actual`) with `use="required"`; `DataConstraintBaseType` restricts it without re-declaring `role`, so every conformant 3.0 data constraint states one of the two values. `MetadataConstraintBaseType` pins `role` to `fixed="Allowed"`, so only **data** constraints can be `Actual`. In 3.1 the attribute is gone entirely (zero occurrences; `ConstraintRoleType` no longer exists), `DataConstraintType`'s documentation reads allowed-only, and the "actual holdings" purpose moved to the 3.1-only `AvailabilityConstraintType` — which is structurally **disjoint** from a 3.0 Actual constraint: it is non-maintainable with a mandatory single attachment (3 targets, each `maxOccurs="1"`), exactly one `CubeRegion`, and counts, whereas a 3.0 Actual constraint is maintainable with an *optional* attachment from a wider target set, 0..2 regions, unbounded `DataKeySet`s, and a `ReleaseCalendar` slot. The model as drawn could not represent a 3.0 `role="Actual"` constraint at all: `AvailabilityConstraint` cannot hold its structure, and the data arm had no role field — a structural failure of ADR-0008 guardrail #1. The guardrail's own rationale ("the semantic distinction between 3.0 and 3.1 is the type discriminant itself, not a dropped attribute") is falsified by the schema: both 3.0 roles live on **one** wire type, so no discriminant between two model types can encode them.

**Decision**: Two clauses, decided together because the second follows from the first.

1. **Role field.** The data constraint gains `role: Option<ConstraintRole>`, with `ConstraintRole = Allowed | Actual` (exhaustive — D-0021, bounded spec-fixed set). `Some` ⟺ the 3.0 wire (where the attribute is required); `None` ⟺ the 3.1 wire (where the attribute cannot occur). The wire has exactly three expressible shapes — 3.0 Allowed, 3.0 Actual, 3.1 no-role — and the store has exactly three states, 1:1 (D-0031). `new()` takes no part: the attribute's requiredness is version-conditional, and the version-agnostic store holds the union; per-version validity is the adapters' concern (ADR-0008 #2/#3). **Mapping record for Actual constraints:** a 3.0 `role="Actual"` constraint maps to `ConstraintModel::Data` with `Some(Actual)` — it **never** maps onto the `Availability` arm, despite the semantic kinship (both express "what data exists"); the structures are disjoint as itemised above.
2. **Rename.** `ReportingConstraint` → `DataConstraint`, matching the spec's `DataConstraintType` in both versions; `ConstraintModel::Data(DataConstraint)`.

**Rationale**: For the field shape: a mandatory `ConstraintRole` with 3.1 parsed as `Allowed` was rejected — unlike `isPartialLanguage` (D-0010), where the schema's own `default="false"` makes absent ⟺ false mechanically identical on re-emission, `role` has no default; the identification is grounded only in 3.1 prose, so the parser would inject a value no wire stated and collapse three expressible shapes into two (the D-0031 information-loss move). Absorbing 3.0-Actual into `AvailabilityConstraint` was rejected on the structural disjointness; a third type or enum variant was rejected because 3.0 models the distinction as an attribute on one type, not as two types (strict adherence to spec). For the rename: D-0002 commits types to map 1-to-1 to named spec concepts; the invented name encoded precisely the allowed-only ("reporting limits") issue — a `role="Actual"` constraint states what data *exists*; and the type's own attachment enum (`DataConstraintAttachment`, D-0034) already carried the spec name. No collision, so no `Sdmx` prefix (D-0027 naming rule).

**Consequences**: (1) Design 0010 swept: §5 narrative, §5.8 blueprint (new `ConstraintRole` enum + `role` field on the renamed `DataConstraint`), the §5.2 validator-table and §5.5 mentions, and the Drawbacks naming entry rewritten as a reversal record. (2) ADR-0008 corrected in place: the attribute is named `role`, not `constraint_type`, and guardrail #1's rationale now cites the role field rather than claiming the discriminant suffices. (3) Cross-version emission — what a 3.1 writer does with `Some(Actual)`, what a 3.0 writer does with `None` — is Phase-2 adapter policy (ADR-0008 #2/#3), deliberately not a model concern; the model's job ends at making all three shapes representable. (4) Provenance class: a 3.0-only superset member, the mirror of `isPartialLanguage` (D-0010, 3.1-only). (5) Earlier entries that reference `ReportingConstraint` (D-0013, D-0032, D-0034, D-0036) gain a rename pointer at first mention; their bodies are otherwise unchanged (audit trail preserved).

---

### D-0038 — Member selections modelled to the full MemberSelectionType; non-empty Values enforced (corrects D-0026's include claim)

> **Amended 2026-06-11 by [D-0051](#d-0051)/[D-0052](#d-0052)**: the node structs now carry their `id` directly and live in ordered `Vec`s (not id-keyed maps), and `include` is stored as `Option<bool>` — the bare-bool exactness argument below treated the schema default as data. The node-struct shape and non-empty enforcement stand.

| **Area**     | Constraints |
| **Phase**    | Phase-1 |
| **Status**   | Active (selection stores amended by [D-0051](#d-0051), [D-0052](#d-0052); three-level-validity enumeration completed by [D-0064](#d-0064)) |
| **Keywords** | constraint, cube-region, member-selection, include, remove-prefix, validity, non-empty, newtype, spec-alignment |
| **Spec ref** | [SDMXStructureConstraint.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureConstraint.xsd#L293-L326) (`MemberSelectionType`, `CubeRegionKeyType`, `ComponentValueSetType`); [3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXStructureConstraint.xsd#L381-L414) (identical for this entire cluster) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.8 |
| **Related**  | [D-0017](#d-0017), [D-0019](#d-0019), [D-0026](#d-0026), [D-0027](#d-0027), [D-0031](#d-0031), [D-0032](#d-0032), [D-0034](#d-0034) |

> **CORRECTED 2026-06-19 by [D-0064](#d-0064).** The Observation's "three-level pattern (region: prohibited; dimension-selection: allowed; per-value: allowed)" understates the validity map by one arm: `TimeRangeValueType` carries its own `validFrom`/`validTo` (`StandardTimePeriodType`), a wrapper-level occurrence this audit did not enumerate. Validity-of-selected-content **forks by choice arm**: per-value on the `Value+` arm ([D-0040](#d-0040)), per-wrapper on the `TimeRange` arm, so it is one tier with two arm-specific realisations, not three flat levels. The selection-node findings (a)/(b)/(c) below otherwise stand; only the completeness of the validity enumeration is corrected. Body retained for provenance.

**Observation**: Three findings on the same selection node, all verified identical in 3.0 and 3.1. (a) `MemberSelectionType` — the abstract base of both selection kinds — declares a per-selection `include` (`xs:boolean`, optional, default `true`), inherited by both `CubeRegionKeyType` and `ComponentValueSetType` (neither restriction prohibits it). D-0026 and the design's §5.8 NOTE asserted the opposite ("include is region-level only"). (b) The same node carries two further attributes: `removePrefix` (`xs:boolean`, optional, **no schema default**; meaningful with `CodelistExtension` prefixes on both selection kinds, and `validFrom`/`validTo` (`StandardTimePeriodType`) inherited by `CubeRegionKeyType` but **prohibited** on `ComponentValueSetType` — making validity a three-level pattern (region: prohibited; dimension-selection: allowed; per-value: allowed). (c) A *chosen* `Value+` arm requires ≥1 value, so an empty `Values` list is mechanically schema-invalid — yet `KeyValueSelection::Values(Vec<CubeValue>)` claimed "non-empty by construction" with nothing enforcing it, and `ComponentSelection::Values(vec![])` was a representational duplicate of `Empty`.

**Decision**: Model the selection node as a wrapper struct per kind, named after the spec complexTypes (cf. `DataConstraint` ← `DataConstraintType`): `CubeRegionKey` ← `CubeRegionKeyType` and `ComponentValueSet` ← `ComponentValueSetType` become the `CubeRegion` collection values. Each carries `selection` (the existing D-0026 choice enums `KeyValueSelection`/`ComponentSelection`, unchanged in shape) plus the selection-level attributes: `include: bool` (schema default `true`, so absent ⟺ true round-trips and a bare bool is verbatim — same reasoning as `is_partial`, D-0032); `remove_prefix: Option<bool>` (**no** schema default, so absent-vs-stated is wire state — D-0031); `valid_from`/`valid_to: Option<SdmxTimePeriod>` (D-0027) on `CubeRegionKey` **only** — `ComponentValueSet` has no such fields, making the prohibited state unrepresentable by omission. The `Values` arms of both enums wrap a new non-empty `CubeValues` newtype (private field, `new()` rejects empty → `Error::EmptyCubeValues`, custom `Deserialize` — the D-0019/D-0034 pattern), so `ComponentSelection::Empty` is the *sole* no-values state, exactly mirroring the wire (`<Component id="X"/>` vs a chosen `Value+` arm). With `include` modelled, `Empty`'s semantics are now exact per the spec's own documentation: `Empty` + `include=true` ⟺ "component present, regardless of value"; `Empty` + `include=false` ⟺ "component absent". The wrapper structs are pub-field carriers with derived `Deserialize` (no cross-field invariant; every field self-enforcing — D-0017/§7); `CubeRegion` keeps its custom structural `Deserialize`.

**Rationale**: The attributes belong on the selection node, not threaded through the choice arms — the spec's two axes (node attributes vs content choice) map to struct fields vs the inner enum. One *shared* `CubeValues` newtype rather than two bespoke ones: the D-0034 bespoke rationale keyed on arms holding *distinct element types*, and both positions currently hold the same `CubeValue` (D-0026). `removePrefix` is carried even though `CodelistExtension` modelling is undecided: it is schema-valid wire data on a modelled node, and the Infoset Store does not condition a field's existence on whether its *referent* is modelled.

**Consequences**: (1) `CubeRegion.key_values`/`components` change value type to the wrapper structs; design §5.8 swept — the erroneous no-per-selection-include NOTE deleted, and the stale "derived Deserialize" comment on `CubeRegion` fixed in passing. (2) D-0026 corrected via blockquote; the structure otherwise stands. (3) `Error` gains `EmptyCubeValues`. (4) Corrects the D-0026 record.

---

### D-0039 — DataKeySet subtree modelled on DataConstraint; 3.1 multi-value keys carried as superset

> **Amended by [D-0051](#d-0051)/[D-0052](#d-0052)**: the key/component selection collections are ordered `Vec`s with ids on the structs (not keyed maps), and the `fixed="true"` `include`s are stored as `Option<bool>` with `Some(false)` rejected — the "not stored" reasoning below collapsed statedness, which the document-integrity contract preserves.
>
> **Mechanism drawn**: the fixed-include rejection's producer now exists — the `FixedInclude` within-field wrapper (custom Deserialize; `new()` rejects a stated `false` with `FixedAttributeMismatch`) carried by `DataKey.include`/`DataKeyValue.include`; the containers stay derived pub-field carriers. The earlier blueprint claimed the check without drawing it.

| **Area**     | Constraints |
| **Phase**    | Phase-1 |
| **Status**   | Active (key stores amended by [D-0051](#d-0051), [D-0052](#d-0052)) |
| **Keywords** | constraint, data-key-set, data-key, superset, divergence, non-empty, newtype, annotable, spec-alignment |
| **Spec ref** | [SDMXStructureConstraint.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureConstraint.xsd#L128-L144) (`DataKeySetType`, `DataKeyType`, `DataKeyValueType`, `DataComponentValueSetType`, `SimpleKeyValueType`, `DataComponentValueType`); [3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXStructureConstraint.xsd#L157-L173) (identical except `DataKeyValueType` — see Observation) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5, §5.8 |
| **Related**  | [D-0011](#d-0011), [D-0026](#d-0026), [D-0031](#d-0031), [D-0033](#d-0033), [D-0034](#d-0034), [D-0036](#d-0036), [D-0037](#d-0037), [D-0038](#d-0038) |

**Observation**: `DataConstraintType` allows `DataKeySet` 0..unbounded in both versions; the model could not express any key set, the official `data_constraint_datakeyset.xml` sample was wholly unrepresentable, and D-0036's own rationale ("a data constraint may be expressed purely via DataKeySets") presumed the structure exists. Verified structure: `DataKeySetType` = `Key` (`DataKeyType`) **1..unbounded** + **required** `isIncluded` (`xs:boolean`, no default). `DataKeyType` is a *restriction of `RegionType`* — a data key is itself a region: **annotable**, two keyed selection collections (`KeyValue` = `DataKeyValueType` 0..\*, `Component` = `DataComponentValueSetType` 0..\*), `include` **`fixed="true"`**, and — unrecorded anywhere before — it does **not** prohibit `RegionType`'s optional `validFrom`/`validTo` (unlike `CubeRegionType`, which does). `DataKeyValueType` is a genuine 3.0↔3.1 **divergence**: 3.0 allows exactly **one** `Value` ("Only a single value can be provided"); 3.1 makes it **unbounded**, with an in-schema annotation documenting the change (multi-value keys, e.g. FREQ = A or M or Q). Both versions: id `SingleNCNameIDType`, `include` fixed `true`, validity prohibited, `removePrefix` inherited. Its element type `SimpleKeyValueType` prohibits *every* attribute (cascade, lang, validity) — bare `xs:string`. `DataComponentValueSetType` mirrors the cube `ComponentValueSetType` (optional choice → the `Empty` state; `include` default `true`; `removePrefix`; validity prohibited) but with `DataComponentValueType` values (cascade + optional `xml:lang`, **no** validity).

**Decision**: Model the subtree; no cut. `DataConstraint` gains `key_sets: Vec<DataKeySet>` (0..unbounded — plain `Vec`, empty valid). `DataKeySet { keys: DataKeys, is_included: bool }` — `DataKeys` is a bespoke non-empty newtype (`Error::EmptyDataKeys`; the 1..unbounded bound is mechanical), and `is_included` is a mandatory `bool` (required attribute: absence is schema-invalid, so there is no absent state to preserve). `DataKey { key_values: BTreeMap<String, DataKeyValue>, components: BTreeMap<String, DataComponentValueSet>, annotations: Vec<Annotation>, valid_from/valid_to: Option<SdmxTimePeriod> }` — annotations per the D-0033 bare-field placement (a non-identifiable annotable type, exactly the case D-0033's forward rule predicted would arise); custom `Deserialize` for the same structural two-collection mapping as `CubeRegion`. `DataKeyValue { values: SimpleKeyValues, remove_prefix: Option<bool> }` — `SimpleKeyValues` is a non-empty `Vec<String>` newtype (`Error::EmptySimpleKeyValues`); values are bare `String`s because `SimpleKeyValueType` is attribute-less `xs:string` (a wrapper struct would invent structure). `DataComponentValueSet { selection: DataComponentSelection, include: bool, remove_prefix: Option<bool> }` with `DataComponentSelection = Values(DataComponentValues) | TimeRange | Empty` (non-empty newtype, `Error::EmptyDataComponentValues`) and `DataComponentValue { value, cascade, lang: Option<String> }` (loose single-tag `lang` per the D-0011 `AnnotationUrl` precedent; no validity fields — prohibited). The `fixed="true"` `include` attributes (on `DataKeyType` and `DataKeyValueType`) are **not stored**: their value space is `{true}` and absent ⟺ stated-true on re-emission, so omission is the spec's own canonicalisation (same class as a schema default — no schema-valid distinction is lost, D-0031-compatible). **Superset divergence record**: the non-empty `Vec` carries 3.1's multi-value keys and covers 3.0's exactly-one; what a 3.0 writer does with `len > 1` is Phase-2 adapter policy (same class as `DataConstraint.role: None` → 3.0, D-0037).

**Rationale**: Model-not-cut is forced by the evidence (official sample + D-0036's presumption). Keyed `BTreeMap`s with the required id as key mirror the `CubeRegion` shape (D-0026/D-0038); bespoke non-empty newtypes with named empty-errors follow D-0019/D-0034; the structure reuses `TimeRange` and `Cascade` unchanged.

**Consequences**: (1) `Error` gains `EmptyDataKeys`, `EmptySimpleKeyValues`, `EmptyDataComponentValues`. (2) Design §5 narrative and §5.8 blueprint gain the subtree; `DataConstraint` field order mirrors the wire (attachment, key sets, regions). (3) `DataKey`'s annotability extends the D-0033 audit count as its forward rule anticipated.

---

### D-0040 — CubeValue split into spec-exact per-value types (CubeKeyValue / SimpleComponentValue) carrying cascade, lang, and validity

> **Amended 2026-06-11 by [D-0052](#d-0052)**: `cascade` is stored as `Option<Cascade>` on every value type (the schema default `false` is an effective view).

| **Area**     | Constraints |
| **Phase**    | Phase-1 |
| **Status**   | Active (cascade statedness amended by [D-0052](#d-0052)) |
| **Keywords** | constraint, cube-region, per-value, validity, xml-lang, cascade, newtype, spec-alignment |
| **Spec ref** | [SDMXStructureConstraint.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureConstraint.xsd#L463-L483) (`SimpleComponentValueType` lines 463–484, `CubeKeyValueType` lines 485–496); [3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXStructureConstraint.xsd#L548-L568) (lines 548–580, identical) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.8 |
| **Related**  | [D-0011](#d-0011), [D-0026](#d-0026), [D-0027](#d-0027), [D-0031](#d-0031), [D-0034](#d-0034), [D-0038](#d-0038), [D-0039](#d-0039) |

**Observation**: The spec types each per-value position as its own restriction of `SimpleComponentValueType`, with a distinct attribute set (identical 3.0/3.1). The full four-member family: cube `Component` values = `SimpleComponentValueType` (`xs:string` content + `cascadeValues` default `false` + optional `xml:lang` + optional `validFrom`/`validTo`); cube `KeyValue` values = `CubeKeyValueType` (prohibits only `xml:lang` — keeps cascade and validity); key-set `Component` values = `DataComponentValueType` (cascade + lang, validity prohibited — D-0039); key-set `KeyValue` values = `SimpleKeyValueType` (everything prohibited — bare string, D-0039). The model's single shared `CubeValue { value, cascade }` (D-0026) dropped per-value validity on both cube kinds and `lang` on the component side; the official `data_constraint_cuberegion.xml` sample exercises per-value `validFrom`/`validTo`. `SimpleComponentValueType` is additionally reused by `MetadataAttributeValueSetType` (metadata constraints — out of 0010 scope, D-0034) and is the restriction base of the other three.

**Decision**: Retire `CubeValue`. The cube positions get spec-exact bespoke types (names are the spec's own complexType names, stripped of `Type` per the established convention): `CubeKeyValue { value, cascade, valid_from, valid_to }` and `SimpleComponentValue { value, cascade, lang: Option<String>, valid_from, valid_to }` — validity as `Option<SdmxTimePeriod>` (D-0027, consistent with D-0038's selection-level fields), `lang` as the loose single-tag `Option<String>` (D-0011 precedent), prohibited attributes unrepresentable by field omission (the D-0038/D-0039 move). The shared `CubeValues` non-empty newtype splits accordingly, per the contingency D-0038 recorded: `CubeKeyValues` (in `KeyValueSelection::Values`) and `SimpleComponentValues` (in `ComponentSelection::Values`), each a bespoke private-field newtype whose `new()` rejects empty — `Error::EmptyCubeValues` is **replaced** by the pair `EmptyCubeKeyValues` / `EmptySimpleComponentValues` (D-0034 named-error pattern). Both value structs remain invariant-free pub-field carriers with derived `Deserialize`.

**Rationale**: Four bespoke types over one maximal type with lint-policed prohibited combinations: the restrictions are *mechanical* (XSD `use="prohibited"`), so a `lang` on a dimension value is schema-invalid wire — exactly what the type should make unrepresentable (D-0026/D-0034 spirit), not hold-and-flag (which D-0031 reserves for schema-*valid* but dubious shapes). Spec-exact names over "more neutral" invented ones per D-0002/strict adherence to spec (verified: both names are real complexType names in both versions, and `CubeKeyValueType` is used in exactly one position).

**Consequences**: (1) `CubeValue` and `CubeValues` are gone; `KeyValueSelection::Values(CubeKeyValues)`, `ComponentSelection::Values(SimpleComponentValues)`. (2) `Error::EmptyCubeValues` (added by D-0038) is replaced by the two named variants — D-0038's text reads with this split applied, as it anticipated. (3) The four-type family is now fully modelled across D-0039 (key-set half) and this entry (cube half). (4) If metadata constraints ever come into scope, `MetadataAttributeValueSetType.Value` reuses `SimpleComponentValue` unchanged — the spec name already covers that position. (5) The `TimePeriodRange.period` raw-`String` alignment follow-up (D-0038 consequence 6) still stands; per-value validity does not change it.

---

### D-0041 — DataConstraint.attachment is Option; the availability attachment stays mandatory

| **Area**     | Constraints |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | constraint, attachment, optionality, spec-alignment |
| **Spec ref** | [SDMXStructureConstraint.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureConstraint.xsd#L53-L69) (`DataConstraintBaseType` `ConstraintAttachment` `minOccurs="0"`; `AvailabilityConstraintType` `ConstraintAttachment` `minOccurs="1"`); [3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXStructureConstraint.xsd#L56-L73) (data side identical) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.8 |
| **Related**  | [D-0031](#d-0031), [D-0034](#d-0034), [D-0037](#d-0037) |

**Observation**: `ConstraintAttachment` is `minOccurs="0"` on `DataConstraintBaseType` in **both** versions, so a schema-valid data constraint may be unattached (its attachment supplied by context, e.g. its registration). The model drew `attachment: DataConstraintAttachment` (non-`Option`), making that wire shape unrepresentable — a Layer-1 store defect. The availability side is the opposite: `AvailabilityConstraintType` declares its attachment `minOccurs="1" maxOccurs="1"`, so the existing non-`Option` field there is correct.

**Decision**: `DataConstraint.attachment: Option<DataConstraintAttachment>`; no `new()` involvement. The data/availability asymmetry is the spec's and is recorded deliberately: optional on `DataConstraint`, mandatory on `AvailabilityConstraint`.

**Consequences**: (1) Design §5 narrative and §5.8 blueprint updated.

---

### D-0042 — ReleaseCalendar (3.0-only) carried on DataConstraint as a superset member

| **Area**     | Constraints |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | constraint, release-calendar, superset, divergence, sdmx-3.0, spec-alignment |
| **Spec ref** | [SDMXStructureConstraint.xsd 3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXStructureConstraint.xsd#L29-L54) (`ConstraintType` `ReleaseCalendar` `minOccurs="0"` line 41, retained by `DataConstraintBaseType` line 68; `ReleaseCalendarType` lines 134–155); 3.1: zero occurrences in any schema |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.8 |
| **Related**  | [D-0010](#d-0010), [D-0031](#d-0031), [D-0034](#d-0034), [D-0037](#d-0037) |

**Observation**: 3.0's `ConstraintType` carries `ReleaseCalendar` (`minOccurs="0"`), retained by the data-side restriction; `ReleaseCalendarType` is three **required** `xs:string` elements — `Periodicity`, `Offset`, `Tolerance` — whose "P7D"-style duration format is stated only in prose, not as a facet. The element has zero occurrences in 3.1. It received neither superset treatment nor a recorded cut, unlike its provenance-class siblings `isPartialLanguage` (D-0010, 3.1-only) and `role` (D-0037, 3.0-only). 3.0 also allows it on metadata constraints, which remain out of 0010 scope (D-0034).

**Decision**: Carry it: `release_calendar: Option<ReleaseCalendar>` on `DataConstraint`, in wire order (attachment, release calendar, key sets, regions). `ReleaseCalendar { periodicity: String, offset: String, tolerance: String }` — all three mandatory (required elements), all unvalidated `String`s: the duration format is prose-only, so a grammar check is lint territory (D-0031), not a construction rejection. Invariant-free pub-field carrier, derived `Deserialize`.

**Rationale**: A cut would discard real 3.0 wire data from the very area ADR-0008 stakes the superset claim on; carrying three strings is minimal overhead. The 3.0-only provenance mirrors D-0037's `role` exactly, including the writer-side consequence.

**Consequences**: (1) What a 3.1 writer does with `Some(..)` is Phase-2 adapter policy (same class as `role: Some(Actual)` — D-0037). (2) The duration-format lint joins the catalogued coherence-lint surface (D-0031, not built).

---

### D-0043 — Counts stored as Option of i32; integer types mirror the XSD value space

| **Area**     | Constraints |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | constraint, availability, counts, xs-int, signedness, verbatim-store, lint, spec-alignment |
| **Spec ref** | [SDMXStructureConstraint.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureConstraint.xsd#L240-L262) (`AvailabilityConstraintType` `seriesCount`/`obsCount`, `xs:int`, optional — lines 258–259; the type is 3.1-only) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.6, §5.8 |
| **Related**  | [D-0022](#d-0022), [D-0027](#d-0027), [D-0031](#d-0031) |

**Observation**: `seriesCount`/`obsCount` are optional `xs:int` — the same wire type as `Dimension.position`, which the model stores as `i32` precisely because "a negative stated position is schema-valid even if meaningless — not ours to reject; a coherence lint flags it". The counts were stored as `Option<u32>`, applying the opposite policy to an identical wire type: a schema-valid negative count became unrepresentable and would be rejected on deserialisation.

**Decision**: `series_count`/`obs_count: Option<i32>`, with a negative-count member added to the catalogued coherence-lint surface. The crate-wide rule, stated once: **the Rust integer type mirrors the XSD value space** — `xs:int` → `i32` (the wire mechanically admits negatives); unsigned types only where the lexical space mechanically excludes a sign (`SdmxVersion`'s components, parsed from a digits-only validated grammar; the `xs:positiveInteger` length facets in `TextFormat`).

**Rationale**: Recording "counts are different because negative counts are meaningless" would be precisely the position rationale's rejected move (meaningless-but-schema-valid is lint territory, not type territory). The value-space rule makes the next `xs:int`-vs-`u32` call mechanical instead of per-field.

**Consequences**: (1) `AvailabilityConstraint.series_count`/`obs_count` change to `Option<i32>`; the stale `cf. position/series_count` aside in §5.2's `SdmxVersion` comment is corrected to cite the rule rather than the now-fixed counterexample. (2) Negative-count lint catalogued (D-0031, not built).

---

### D-0044 — 3.0-only data-source attachment members modelled (SimpleDataSource arm; QueryableDataSource companions)

> **Field names renamed 2026-07-08:** `is_rest_datasource`/`is_web_service_datasource` → `is_rest_data_source`/`is_web_service_data_source`, following Rust word boundaries (the wire attributes stay `isRESTDatasource`/`isWebServiceDatasource`). A name sync, not a new decision, so the reference below is updated in place.

| **Area**     | Constraints |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | constraint, attachment, data-source, superset, divergence, sdmx-3.0, non-empty, spec-alignment |
| **Spec ref** | [SDMXStructureConstraint.xsd 3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXStructureConstraint.xsd#L277-L303) (`DataConstraintAttachmentType` lines 277–304); [SDMXCommon.xsd 3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXCommon.xsd#L767-L798) (`QueryableDataSourceType` line 767); 3.1: both removed from constraint attachments (`QueryableDataSourceType` survives in 3.1 Common for registry use only) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5, §5.8 |
| **Related**  | [D-0014](#d-0014), [D-0031](#d-0031), [D-0034](#d-0034), [D-0037](#d-0037), [D-0042](#d-0042) |

**Observation**: 3.0's `DataConstraintAttachmentType` is wider than the four reference targets D-0034 recorded (its count was 3.1-accurate only): it additionally allows a `SimpleDataSource` choice arm (`xs:anyURI`, 1..unbounded — URLs of SDMX-ML data/metadata messages) and trailing `QueryableDataSource` elements (`common:QueryableDataSourceType`, 0..unbounded) inside each of the DataStructure/Dataflow/ProvisionAgreement sequences. 3.1 removed both from constraint attachments entirely (the 3.1 abstract `ConstraintAttachmentType` holds only the 8 reference targets). `QueryableDataSourceType` = `DataURL` (`xs:anyURI`, required) + optional `WSDLURL`/`WADLURL` + **required** `isRESTDatasource`/`isWebServiceDatasource` booleans. A 3.0 data constraint attached to a data source was unrepresentable.

**Decision**: Model both as 3.0-only superset members (the D-0037/D-0042 provenance class). `DataConstraintAttachment` gains a `SimpleDataSource(SimpleDataSources)` arm — bespoke non-empty newtype over `Vec<String>` (chosen arm is 1..\*, mechanical; URLs unvalidated `xs:anyURI` per D-0014; `Error::EmptySimpleDataSources`). The three 1..\* reference arms become struct variants carrying their companions: `DataStructure { refs: DataStructureRefs, queryable: Vec<QueryableDataSource> }` (likewise `Dataflow`, `ProvisionAgreement`) — `queryable` empty ⟺ absent (`minOccurs="0"` unbounded; always empty on 3.1 wire). `QueryableDataSource { data_url, wsdl_url, wadl_url, is_rest_data_source, is_web_service_data_source }` is an invariant-free pub-field carrier (both bools mandatory — required attributes). `AvailabilityConstraintAttachment` is untouched (3.1-only type; no data-source members).

**Rationale**: Struct variants because the spec nests refs and queryable sources in one sequence per arm — a separate parallel field would detach them from the arm they belong to. Superset-not-cut for the same reason as `ReleaseCalendar` (D-0042): real 3.0 wire on the type ADR-0008 stakes its claim on.

**Consequences**: (1) D-0034 carries an amendment note. (2) `Error` gains `EmptySimpleDataSources`. (3) What a 3.1 writer does with data-source content is Phase-2 adapter policy (D-0037 class). (4) The metadata-side analogues (`MetadataSet`, `SimpleDataSource` on the metadata attachment) stay out of scope with metadata constraints (D-0034 boundary).

---

### D-0045 — 3.1-only DimensionConstraint (Dataflow) and evolvingStructure (DSD) carried as superset members

> **Item-validation clause amended 2026-07-08 by [D-0077](#d-0077)**: the "structural-only validation per D-0020, not NCName-checked" clause below is superseded — `DimensionConstraint` items now validate `IDType` at construction (the element's own tier, so still correctly not NCName). The carriage decision and the non-empty invariant stand.
>
> **Amended 2026-06-11 by [D-0052](#d-0052)**: `evolving_structure` is stored as `Option<bool>` — the bare-bool exactness argument below treated the schema default as data; XSD defaulting is a view.

| **Area**     | Data structure |
| **Phase**    | Phase-1 |
| **Status**   | Active (statedness storage amended by [D-0052](#d-0052); item validation amended by [D-0077](#d-0077)) |
| **Keywords** | dataflow, dsd, dimension-constraint, evolving-structure, superset, divergence, sdmx-3.1, spec-alignment |
| **Spec ref** | [SDMXStructureDataflow.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureDataflow.xsd#L19-L34) (`DataflowBaseType.DimensionConstraint` 0..1; `DimensionConstraintType` = `Dimension` `common:IDType` 1..unbounded); [SDMXStructureDataStructure.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureDataStructure.xsd#L39-L59) (`DataStructureType.evolvingStructure`, `xs:boolean`, default `false`); 3.0: zero occurrences of either |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.6, §5.7; |
| **Related**  | [D-0010](#d-0010), [D-0019](#d-0019), [D-0020](#d-0020), [D-0031](#d-0031), [D-0034](#d-0034) |

**Observation**: Two coupled 3.1 additions (the official `dataflow_dimensionconstraint.xml` sample exercises the first). `evolvingStructure` flags a DSD that may gain new dimensions under a **minor** version update; `DimensionConstraint` on a Dataflow pins the dimension subset the dataflow uses, which the XSD prose says is required when the dataflow references an evolving DSD with a wildcarded minor version. That requiredness is **prose** (an annotation, not a facet), so it is lint territory (D-0031), never a construction rejection.

**Decision**: Carry both (3.1-only superset members, the `isPartialLanguage` provenance class — D-0010). `Dataflow.dimension_constraint: Option<DimensionConstraint>` where `DimensionConstraint` is a bespoke non-empty newtype over `Vec<String>` (`Dimension` is 1..unbounded — mechanical; `Error::EmptyDimensionConstraint`; the ids are *references* to dimensions, so structural-only validation per D-0020, not NCName-checked). `DataStructureDefinition.evolving_structure: bool` — a bare bool preserves the infoset exactly: absent ⟺ `false` in 3.1 (the schema's own default) and 3.0 cannot state it at all (⟺ `false`); contrast `isMultiLingual`, whose *flipped* default forces `Option` when speced ([D-0046](#d-0046)).

**Consequences**: (1) Design §5.6/§5.7 swept; `Error` gains `EmptyDimensionConstraint`. (2) The evolving-DSD/dimension-constraint coupling rule joins the catalogued lint surface (D-0031, not built). (3)`evolving_structure` is addressed through the validated `new()`.

---

### D-0046 — 3.0↔3.1 divergences resolved by carrying the superset; the disposition table is the reconciliation baseline

> **Parent-row grounds amended 2026-07-08 by [D-0077](#d-0077).** The `Item.Parent`/`Code.Parent`/`Concept.Parent` divergence row below is carried "no-op — `parent_id: Option<String>` is structurally validated only (D-0020)". D-0077 validates stated parents at the union of the two editions' tiers (`Code.parent_id` `IDType`; `Concept.parent_id` `NCNameIDType`, the two editions' identical pattern), so the row's carrying ground is now union-tier validation, no longer non-validation. The disposition itself (carry the superset; never version-branch) is unchanged.

| **Area**     | Architecture |
| **Phase**    | Phase-1 |
| **Status**   | Active (Parent-row grounds amended by [D-0077](#d-0077)) |
| **Keywords** | divergence, superset, carry, reconciliation-baseline, spec-alignment |
| **Spec ref** | All in-scope pinned XSDs: `SDMXCommon`, `SDMXCommonReferences`, `SDMXStructureBase`, `SDMXStructureCodelist`, `SDMXStructureConcept`, `SDMXStructureOrganisation`, `SDMXStructureDataStructure`, `SDMXStructureDataflow`, `SDMXStructureConstraint`, `xml.xsd` (both `specs/3.0/schemas/` and `specs/3.1/schemas/`) |
| **Source**   | [ADR-0008](adr/0008-model-sdmx-3-0-and-3-1-divergence-with-a-unified-constraintmodel.md) (unified superset guardrail); 0010 quality assessment |
| **Related**  | [D-0037](#d-0037), [D-0039](#d-0039), [D-0042](#d-0042), [D-0044](#d-0044), [D-0045](#d-0045) |

**Observation**: 3.0↔3.1 divergences had been discovered ad hoc, leaving open whether the divergence set was complete and whether each member had a consistent resolution. A systematic pass settled the question: a normalised structural diff (annotations, namespace and formatting boilerplate stripped) over every in-scope schema, plus a manual sweep of `SDMXStructureConstraint.xsd`, enumerated the full set. `SDMXStructureOrganisation.xsd` and `xml.xsd` have **no divergence at all** (so the `Contact` gap is a both-versions issue, not a divergence); the complete divergence set over the rest is the table below.

**Decision**: Every divergence touching a modelled type is resolved by **carrying the superset** — the type set holds the union of what 3.0 and 3.1 can express, each version-specific member tagged with its provenance — never by version-branching the type or dropping a side. This is the per-type application of ADR-0008's unified-superset guardrail. The disposition table records the complete divergence set and how each member is carried, ruled a no-op, or routed out of scope; each "carried" row delegates to the entry that draws it. The table is the **reconciliation baseline**: any future schema re-pinning re-runs the normalised-diff method and reconciles its result against this set.

| Divergence | Disposition |
|---|---|
| `ConstraintType.role` required in 3.0, absent in 3.1 | carried — [D-0037](#d-0037) |
| `ReleaseCalendar` 3.0-only | carried — [D-0042](#d-0042) |
| `AvailabilityConstraintType` + its attachment 3.1-only | carried — D-0013/D-0033/D-0036 |
| `DataKeyValueType` value: single (3.0) → unbounded (3.1) | carried — [D-0039](#d-0039) |
| `isPartialLanguage` 3.1-only | carried — D-0010 |
| `SimpleDataSource` arm + `QueryableDataSource` companions 3.0-only | carried — [D-0044](#d-0044) |
| `DimensionConstraint` + `evolvingStructure` 3.1-only | carried — [D-0045](#d-0045) |
| `TextFormatType.isMultiLingual` default: `true` (3.0) → `false` (3.1) | carried — [D-0048](#d-0048)/[D-0052](#d-0052); the flipped default means absent has version-dependent meaning, so the facet lands as `Option<bool>` (no bare-bool collapse available). The same family check exposed a non-divergent design error: `pattern` is optional on the *uncoded* `TextFormatType` (both versions), which the design wrongly stated is coded-only — also corrected by [D-0048](#d-0048) |
| `Item.Parent` `NestedIDType`→`IDType`; `Code.Parent` `SingleNCNameIDType`→`IDType`; `Concept.Parent` `SingleNCNameIDType`→`NCNameIDType` | no-op — `parent_id: Option<String>` is structurally validated only (D-0020), which carries both sides; note D-0023's observation table documents the 3.1 state of the Parent column |
| `SemanticVersionReferenceType` wildcard pattern: 3.0 admits `major+`, 3.1 does not (`VersionType` itself identical, doc typos aside) | no-op now — reference `version` fields are raw `String`s; the divergent wildcard grammar is in scope for the Phase-2 reference-types/URN-contract entry gate (ROADMAP Phase 2 → Parsers) |
| `ActionType` gains `Merge` (3.1); `*OrNotApplicableType`/`*OrMissingType` sentinel unions (3.1-only); one Categorisation URN tightening; metadata-side `isMultiLingual` reach | out of 0010 scope (message/data/registry/category surfaces — Phase 2+) |

**Rationale**: Carrying the superset over version-branching is what ADR-0008 commits the crate to — one type set both versions deserialize into, so a 3.0 document and a 3.1 document of the same artefact produce the same Rust type, and a value absent in one version is simply an unstated optional member rather than a type the consumer must match on by version. The systematic diff (rather than continued ad-hoc discovery) is what lets the rule be asserted as *exhaustive*: every divergence is accounted for, so "carried the superset" is a closed claim, not an aspiration.

**Consequences**: (1) The disposition table is the reconciliation baseline for future spec updates — re-pinning reconciles against it. (2) The provenance classes named here (3.0-only, 3.1-only, default-flip) recur as the vocabulary later entries cite when carrying a version-specific member.

---

### D-0047 — ValueList modelled as a maintainable artefact (not an item scheme); items are an ordered Vec

| **Area**     | Codelist |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | value-list, maintainable, fourth-id-tier, duplicates, ordered, annotable, spec-alignment |
| **Spec ref** | [SDMXStructureCodelist.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureCodelist.xsd#L321-L336) (`ValueListBaseType`/`ValueListType`/`ValueItemType`, lines 321–370); [3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXStructureCodelist.xsd#L316-L331) (identical, doc typos aside); [SDMXCommonReferences.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXCommonReferences.xsd#L870-L878) (`AnyCodelistReferenceType` — Codelist or ValueList) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.5 |
| **Related**  | [D-0023](#d-0023), [D-0031](#d-0031), [D-0033](#d-0033), [D-0048](#d-0048) |

**Observation**: `ValueList` is a real artefact in both versions, referenceable from concept/attribute/measure representations (`AnyCodelistReferenceType`), so the boundary could not be drawn silently. Verified structure: `ValueListBaseType` restricts `MaintainableType` (so a ValueList is maintainable: agency/id/version, `Name+` **required**; its id is **not** NCName-tightened, staying `IDType` — unlike `Codelist`); `ValueListType` holds `ValueItem` **0..unbounded** (an empty list is schema-valid). `ValueItemType` is **not** an `ItemType`: it extends `AnnotableType` directly, its `Name` is **optional** (`minOccurs="0"`), and its `id` is plain **`xs:string`** (`use="required"`) — a fourth identifier tier beyond D-0023's three (unrestricted: `$`, `€`, `¥`, even an empty string are mechanically valid). The XSDs declare **no uniqueness** over ValueItem ids, and the official `valuelist.xml` sample contains a duplicate (`¥` twice, for CNY and JPY).

**Decision**: Model the artefact in full (review ruling: full-model objective before implementation). `ValueList { metadata: MaintainableMetadata, items: Vec<ValueItem> }` — a pub-field carrier with derived `Deserialize` (no invariant of its own: empty items is schema-valid, the metadata enforces itself, and the id needs no tighten). `ValueItem { id: String, names: Option<LocalisedString>, descriptions: Option<LocalisedString>, annotations: Vec<Annotation> }` — `id` deliberately **unvalidated** (the fourth tier: any `xs:string` is mechanically valid, so there is nothing for Layer 1 to reject); `names`/`descriptions` are `Option<LocalisedString>` (zero names ⟺ `None`; the non-empty `LocalisedString` invariant covers the ≥1 case exactly); `annotations` per the D-0033 bare-field placement (annotable, non-identifiable-typed). **Items are a `Vec`, not a `BTreeMap`**: duplicate ids are schema-valid wire that official material actually exhibits, so identity-keyed storage would silently destroy content — the Infoset Store holds the element list verbatim, order included. `ValueList` joins the `SdmxSerialize` sealed list and gains `ValueListReference { agency, id, version }` (maintainable triple, like `CodelistReference`) for the representation positions (D-0048).

**Rationale**: `ItemScheme<I>` is structurally wrong for it (its items must be `IdentifiableArtefact`s with validated ids and required names; `ValueItem` is none of those), so a bespoke type is not a divergence from the framework but the spec's own shape. The `Vec` choice is *not* a decision on thekeyed-collection question for genuine item schemes — there the ids are validated identity keys; here they are explicitly unconstrained strings with duplicate usage in published material.

**Consequences**: (1) D-0023's three-tier table gains a recorded fourth tier (unrestricted `xs:string`, ValueItem only) — no new validator exists, by design. (2) The design's Summary/scope list gains Value Lists. (3) `ValueList` enters §5.10's sealed serialisation list. (4) Duplicate-id and blank-id *quality* concerns are lint territory (D-0031), catalogued not built. (5) Unblocks D-0048's `EnumerationReference`.

---

### D-0048 — Representation subsystem completed: superset store + per-position constructor enforcement

> **Amended 2026-06-11 by [D-0052](#d-0052)**: `min_occurs` is stored as `Option<u32>` and the textTypes as `Option<DataType>` (defaults are effective views, position-aware for the time tier); `is_multi_lingual`'s `Option<bool>` below is now the general statedness rule, not a flip-forced exception.
>
> **Corrected 2026-06-11** (type name further corrected 2026-06-12): the "(no default → `Option`)" claim for `maxOccurs` below holds only at the base tier — `AttributeRepresentationType` and `MeasureRepresentationType` re-declare it `default="1"` (both versions; the dimension position prohibits it). (There is no type named `BasicComponentRepresentationType`; the real re-declarers are `AttributeRepresentationType` at `SDMXStructureDataStructure.xsd` 3.1:573 and `MeasureRepresentationType` at :591. `ConceptRepresentation` inherits the base `RepresentationType` with no `maxOccurs` default, so this is a per-position property, not a Basic-tier one.) The `Option<MaxOccurs>` store is right; the applied default is a position-aware `effective_max_occurs()` view, the same shape as `text_type`'s.

| **Area**     | Data structure |
| **Phase**    | Phase-1 |
| **Status**   | Active (facet statedness amended by [D-0052](#d-0052)) |
| **Keywords** | representation, enumeration, value-list, text-format, facets, is-multi-lingual, pattern, occurs, constructor, spec-alignment |
| **Spec ref** | [SDMXStructureBase.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureBase.xsd#L209-L242) (`RepresentationType`, the TextFormat tier chain, `CodeDataType`); [SDMXStructureDataStructure.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureDataStructure.xsd) (the four concrete representation types); [SDMXStructureConcept.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureConcept.xsd#L101-L120) (`ConceptRepresentation`); [SDMXCommon.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXCommon.xsd#L1244-L1470) (`DataType` subsets); 3.0 identical throughout except the `isMultiLingual` default ([D-0046](#d-0046)) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.6.1 |
| **Related**  | [D-0023](#d-0023), [D-0028](#d-0028), [D-0029](#d-0029), [D-0031](#d-0031), [D-0038](#d-0038), [D-0046](#d-0046), [D-0047](#d-0047) |

**Observation**: The verified per-position table (identical 3.0/3.1): concept core / attribute / measure use `BasicComponentTextFormatType` (textType ⊆ Basic, 41 of 44; `isMultiLingual` and `pattern` allowed) with `Enumeration` of `AnyCodelistReferenceType` (**Codelist or ValueList**); dimension uses `SimpleComponentTextFormatType` (textType ⊆ Simple, 40; `isMultiLingual` **prohibited**) with **Codelist-only** enumeration and representation-level `maxOccurs` **prohibited**; TimeDimension uses `TimeTextFormatType` (only `textType` ⊆ Time (17) / `startTime` / `endTime`; **no enumeration at all**). `EnumerationFormat` (`CodedTextFormatType`) is position-uniform with textType ⊆ Code (33). Subset deltas: Basic = DataType − {DataSetReference, IdentifiableReference, KeyValues}; Simple = Basic − {XHTML}; Code = Simple − {DateTime, Decimal, Double, Float, GeospatialInformation, Time, TimeRange}. No modelled position uses the bare 15-attribute `TextFormatType`. The model as drawn could not reference a ValueList anywhere, enforced none of the position restrictions, mis-stated `pattern` as coded-only, omitted `isMultiLingual`, and promised but never drew the representation-level occurs attributes (whose `maxOccurs` is `OccurenceType` — a number **or** `"unbounded"`, unrepresentable in the promised `Option<u32>`).

**Decision** (review ruling: superset store + constructor enforcement, not per-position struct multiplication). *Store*: `Representation` becomes a wrapper struct (the D-0038 node idiom) — `{ choice: RepresentationChoice, min_occurs: u32, max_occurs: Option<MaxOccurs> }`, where `min_occurs` is a bare `u32` (schema default 1, absent ⟺ 1 — D-0032 reasoning; u32 per the D-0028 length narrowing), `MaxOccurs = Count(u32) | Unbounded` (no default → `Option`; the literal gets its own arm), and `RepresentationChoice = Enumeration { enumeration: EnumerationReference, format: Option<EnumerationFormat> } | TextFormat(TextFormat)` with `EnumerationReference = Codelist(CodelistReference) | ValueList(ValueListReference)` ([D-0047](#d-0047)). `TextFormat` gains `pattern: Option<String>` (correcting the coded-only claim) and `is_multi_lingual: Option<bool>` (**`Option` is mandatory, not style**: the schema default flips 3.0 `true` → 3.1 `false` — D-0046 — so absent has version-dependent meaning). `DataType` stays the single wide 44-value enum (the store), with subset-membership predicates as views. *Enforcement*: the per-position mechanical restrictions are checked in the component constructors — the D-0023 pattern (the type that knows its position owns the check): `Dimension::new()` rejects a ValueList enumeration, a stated `is_multi_lingual`, a stated `max_occurs`, and a textType outside Simple; `TimeDimension::new()` rejects any `Enumeration` and any facet outside {Time-subset textType, `start_time`, `end_time`}; `Attribute`/`Measure`/`Concept` reject textType outside Basic; every enumerated position rejects an `EnumerationFormat` textType outside Code. `Error` gains `ValueListEnumerationNotAllowed(String)`, `EnumerationNotAllowed(String)`, `ProhibitedRepresentationFacet(String, String)`, `InvalidTextTypeForComponent(String, String)`.

**Rationale**: Constructor enforcement keeps one `Representation` vocabulary for consumers while making every mechanically schema-invalid combination unconstructible — otherwise five parallel representation structs (plus three TextFormat structs and four DataType enums) would triple the surface to encode what are checks over the same fields. The store stays a superset (Layer 1 holds everything any position can express); position validity is the component's own invariant, exactly like its tightened id.

**Consequences**: (1) D-0028 carries an amendment note (four corrections). (2) The §5.6.1 "carried on the component wrapper" occurs-drift is closed by drawing the fields on the `Representation` wrapper instead. (3) The textType subset predicates are Layer-2 views used by Layer-1 checks; a future spec value lands in `DataType` once and the predicates updated deliberately (D-0021 exhaustive-enum reasoning). (4) Completes the D-0046 handover (`isMultiLingual`, uncoded `pattern`).

---

### D-0049 — DSD container redrawn: identifiable descriptor structs; the DSD itself becomes a derived carrier

> **`GroupDimensions` refs amended 2026-07-08 by [D-0077](#d-0077)**: the "refs structural-only per D-0020" clause below is superseded — each `GroupDimensions` item now validates `NCNameIDType` at construction (the `DimensionReference` element's tier). The descriptor redraw, the non-empty invariant, and the `Vec<Group>` store stand.
>
> **Amended 2026-06-11 by [D-0051](#d-0051)/[D-0052](#d-0052)**: descriptor contents are ordered `Vec`s (`AttributeList` holds a single interleaved member `Vec` — the wire is one repeated choice; `attributes()`/`usages()` are filtered views), and the fixed descriptor ids are stored as `Option<String>` with mismatch rejected (statedness), rather than omitted as below.
>
> **Corrected 2026-06-12 (schema-fidelity pass).** The "Group ids carry no `xs:unique` (duplicates are schema-valid)" claim in the Observation/Decision/Rationale below is **false against the XSD**. `DataStructureUniqueComponent` (`SDMXStructureDataStructure.xsd` 3.1:65 / 3.0:53) is an `xs:unique` whose selector lists `structure:Group | …/Dimension | …/TimeDimension | …/Attribute | …/ReportingYearStartDay | …/Measure` on field `@id`; `Group @id` is `use="required"` (`GroupBaseType` ~3.1:431), so explicit duplicate group ids are **schema-invalid**. `Vec<Group>` still stands, justified now by **wire-order preservation** (a keyed map sorts) plus the genuine residue the constraint cannot see: an id a component **inherits from its concept identity** escapes XML validation (the `DataStructureComponents` annotation states such checks fall "outside of the XML validation"). Catalogued lint #4 is re-scoped from "duplicate group ids" to that concept-inherited residue.

| **Area**     | Data structure |
| **Phase**    | Phase-1 |
| **Status**   | Active (descriptor stores amended by [D-0051](#d-0051), [D-0052](#d-0052); GroupDimensions refs amended by [D-0077](#d-0077)) |
| **Keywords** | dsd, descriptor, group, dimension-list, attribute-list, measure-list, identifiable, construction-contract, vec-vs-map, spec-alignment |
| **Spec ref** | [SDMXStructureDataStructure.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureDataStructure.xsd#L81-L95) (`DataStructureComponentsType`, `DimensionListType`, `GroupType`/`GroupBaseType`/`GroupDimensionType`, `AttributeListType`, `MeasureListType`); [SDMXStructureBase.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureBase.xsd#L130-L143) (`ComponentListType` extends `IdentifiableType`); 3.0 identical throughout ([D-0046](#d-0046)) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §4, §5.6, §7 |
| **Related**  | [D-0017](#d-0017), [D-0019](#d-0019), [D-0025](#d-0025), [D-0029](#d-0029), [D-0033](#d-0033), [D-0039](#d-0039), [D-0045](#d-0045), [D-0047](#d-0047), [D-0050](#d-0050) |

**Observation**: Three interlocking issues. (1) The DSD blueprint (pub fields, derived `Deserialize`, no `new()`) contradicted the §7 contract, leaving `EmptyDimensionList` producer-less. (2) The DSD had no `groups`, yet `AttributeRelationship::Group(GroupId)` referenced the structure it could not declare (the ECB_EXR sample uses one). (3) The component containers are **identifiable**: `ComponentListType` extends `IdentifiableType`, so `DimensionList`/`Group`/`AttributeList`/`MeasureList` each carry annotations/links/urn and an id — **fixed** (`"DimensionDescriptor"`/`"AttributeDescriptor"`/`"MeasureDescriptor"`) on the three lists, **required and user-chosen** (`IDType`) on `Group` — all flattened away by the bare collections. Verified content models: `DataStructureComponents` = `DimensionList` (exactly 1) , `Group*`, `AttributeList?`, `MeasureList?`; `DimensionListType` = `Dimension+ TimeDimension?`; `MeasureListType` = `Measure+`; `AttributeListType` = a choice **1..unbounded** of `Attribute` | `MetadataAttributeUsage` ([D-0050](#d-0050)); `GroupType` = required id + `GroupDimension+` (each a `DimensionReference`, NCName). So every *present* list is mechanically non-empty, and "no measures/attributes" is the *absent* list. Group ids carry no `xs:unique` (duplicates are schema-valid).

**Decision** (review rulings: descriptor structs with identity; `Vec<Group>`). Model the descriptors as structs named after the spec types: `DimensionList { annotations, links, urn, dimensions (private, ≥1), time_dimension }`, `AttributeList { annotations, links, urn, attributes: BTreeMap (private), usages: Vec }`, `MeasureList { annotations, links, urn, measures: BTreeMap (private) }`, `Group { metadata: IdentifiableMetadata, dimensions: GroupDimensions }` (a pub-field carrier — both fields self-validate; `GroupDimensions` is the bespoke non-empty refs newtype, `Error::EmptyGroupDimensions`, refs structural-only per D-0020). The three fixed descriptor ids are **not stored** (value space of one — the D-0039 fixed-attribute rule). Each list descriptor owns its mechanical non-empty invariant in a validated `new()` taking the initial collection (`Error::EmptyDimensionList` — its promised producer, relocated; `EmptyAttributeList`; `EmptyMeasureList`) and exposes the §7 `push()` surface. The DSD becomes `{ metadata, dimension_list: DimensionList, groups: Vec<Group>, attribute_list: Option<AttributeList>, measure_list: Option<MeasureList>, evolving_structure }` — `None` ⟺ the wire's absent list (superseding D-0025's no-`Option` clause), and `TimeDimension?` now lives on the descriptor (D-0029 placement refined). The non-empty invariant moved *into* `DimensionList::new()`, making it within-field from the DSD's perspective — so by §7's own strict rule (derived `Deserialize` is correct when every field enforces its own invariants) the DSD is a pub-field **derived** carrier, and §7's listing of the DSD in the custom-impl category was the outdated section; §7 is swept accordingly. `groups` is a **`Vec`**, not a map: no `xs:unique` means duplicate group ids are schema-valid wire a keyed map would silently collapse; `DataStructureDefinition::get_group(&str)` is the Layer-2 lookup view that serves `AttributeRelationship::Group` resolution.

**Rationale**: The descriptors are real, identifiable, annotable wire structure; flattening them violated the D-0033 principle. Invariant placement follows D-0019 (the type owning the invariant enforces it). On `Vec<Group>`: group ids carry no `xs:unique`, so duplicate ids are schema-valid wire a keyed map would silently collapse, and `Group` has no incumbent map decision — the verbatim `Vec` is the conservative per-type call.

**Consequences**: (1) D-0025's no-`Option` clause superseded (note added); D-0033's forward rule amended (note added). (2) §7 swept: the invariant-examples list, the cross-field bullet (descriptors replace the DSD), and the `insert()` sentence (descriptors + `ItemScheme`). (3) `Error` gains `EmptyGroupDimensions`/`EmptyAttributeList`/`EmptyMeasureList`; `EmptyDimensionList` reworded to its real producer. (4) `AttributeRelationship::Group` no longer dangles); duplicate group ids are a catalogued lint (first-match lookup documented). (5) The descriptors are in-scope annotable types with homes — consistent with the amended D-0033 rule.

---

### D-0050 — MetadataAttributeUsage and MeasureRelationship modelled on the attribute list

> **Validation clauses amended 2026-07-08 by [D-0077](#d-0077)**: `MetadataAttributeUsage` is no longer an invariant-free pub-field carrier — it crosses to an invariant-bearing type whose `new()` validates `metadata_attribute_ref` as `NCNameIDType`; `MeasureRelationship` items likewise validate `NCNameIDType` (its non-empty list invariant stands). The modelling decision — both members drawn, the single-link quirk, the `Vec` of usages — is otherwise unchanged.

| **Area**     | Data structure |
| **Phase**    | Phase-1 |
| **Status**   | Active (validation clauses amended by [D-0077](#d-0077)) |
| **Keywords** | attribute-list, metadata-attribute-usage, measure-relationship, component, spec-alignment |
| **Spec ref** | [SDMXStructureDataStructure.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureDataStructure.xsd#L123-L135) (`AttributeListType` choice; `MetadataAttributeUsageBaseType`/`MetadataAttributeUsageType`; `AttributeType.MeasureRelationship`, `MeasureRelationshipType`); 3.0 identical ([D-0046](#d-0046)) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.6 |
| **Related**  | [D-0011](#d-0011), [D-0012](#d-0012), [D-0020](#d-0020), [D-0033](#d-0033), [D-0034](#d-0034), [D-0049](#d-0049) |

**Observation**: Both members sit squarely inside the modelled DSD (the D-0034 metadata-constraint boundary does not cover them). `AttributeListType` is a choice (1..unbounded) of `Attribute` | `MetadataAttributeUsage`. `MetadataAttributeUsageType` is a `ComponentType` restriction whose own documentation explains the shape: it *uses* a metadata attribute already defined in the metadata structure the DSD references, so id is **prohibited** and ConceptIdentity/LocalRepresentation are excluded (`maxOccurs="0"`); what remains is `MetadataAttributeReference` (an NCName **local** reference) plus a full `AttributeRelationship`, with `Annotations` kept and — a mechanical quirk — at most **one** `Link` (`minOccurs="0"`, `maxOccurs` defaulting to 1, unlike the unbounded `Link` everywhere else). `MeasureRelationshipType` on `AttributeType` (0..1) is `Measure` (NCName local ref) **1..unbounded** — the measures an attribute applies to.

**Decision** (review ruling: model both, as drawn). `MetadataAttributeUsage { metadata_attribute_ref: String, relationship: AttributeRelationship, annotations: Vec<Annotation>, link: Option<Link> }` — an invariant-free pub-field carrier; `Option<Link>` because a second link is mechanically schema-invalid here (a `Vec` would over-admit); the ref is structural-only (D-0020 — it points into an MSD artefact that remains outside 0010 scope, which the reference does not require modelling). Usages live as `Vec<MetadataAttributeUsage>` on `AttributeList` (no id to key by; repetition of the same ref is schema-valid). `Attribute` gains `measure_relationship: Option<MeasureRelationship>`, where `MeasureRelationship` is the bespoke non-empty newtype over `Vec<String>` (`Error::EmptyMeasureRelationship`; D-0034 pattern; refs structural-only).

**Rationale**: Model-not-cut: both are real wire inside the DSD's attribute list, and the official-sample-backed superset claim covers the DSD. The usage type's exclusions are encoded by omission (no id/concept/representation fields — the D-0038/D-0039 move); its single-link quirk is mechanical and therefore typed.

**Consequences**: (1) `AttributeList::new()`/`insert_usage()` handle the second choice kind ([D-0049](#d-0049)). (2) `Error` gains `EmptyMeasureRelationship`. (3) `Attribute::new()` gains the parameter; the newtype composes, so no new constructor check. (4) Metadata structures (MSDs) themselves remain out of scope — the local reference does not pull them in.

---

### D-0051 — Wire collections stored as ordered Vecs; identity-keyed maps superseded

> **Corrected 2026-06-12 (schema-fidelity pass).** The original Spec-ref and the Observation below claim "the only `xs:unique` in the 3.1 set covers Category and MetadataAttribute", which is **false**: `grep -c xs:unique specs/3.1/schemas/SDMXStructure.xsd` = **128** (3.0 = 124), and in-scope collections ARE protected. `Codelist_UniqueCode` (:552), `ConceptScheme_UniqueConcept` (:535), `AgencyScheme_UniqueAgency` (:462), `DataStructureUniqueComponent` (`SDMXStructureDataStructure.xsd` 3.1:65), and `DataConstraint_/AvailabilityConstraint_CubeRegionInclusion` (:586/:603) all enforce `@id`/`@include` uniqueness. The **`Vec`-everywhere decision stands**, but on corrected grounds: the universal justification is **wire-order preservation** (a `BTreeMap` sorts), with duplicate-identity preservation a **residual** concern confined to the collections the schema genuinely does not constrain: `ValueItem` ids, concept-inherited DSD component ids, `LocalisedString` languages, and cube-region selection ids. The "hybrid policy collapses to Vec everywhere" conclusion holds for that reason (wire-order preservation), not for the absent-`xs:unique` reason stated below. The Spec-ref cell is corrected in place; the Observation prose is retained for provenance.

| **Area**     | Collections |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | collections, vec, ordering, duplicates, verbatim-store, views, spec-alignment |
| **Spec ref** | All in-scope XSDs (128 `xs:unique` in 3.1 `SDMXStructure.xsd`): explicit `@id` IS uniqueness-enforced for codes/concepts/agencies/DSD-components/groups and `@include` for cube-region direction; the residual uncovered cases (concept-inherited component ids, `ValueItem` ids, `LocalisedString` languages, region key-component "once" rule) are why ordered `Vec` storage is needed, with order preservation as the universal driver |
| **Source**   | [ADR-0023](adr/0023-two-layer-infoset-store-and-derived-views-architecture.md); |
| **Related**  | [D-0006](#d-0006), [D-0031](#d-0031), [D-0047](#d-0047), [D-0049](#d-0049), [D-0052](#d-0052) |

**Observation**: Identity-keyed `BTreeMap` storage destroys two classes of schema-valid wire state: element **order** (always — iteration is sorted, erasing the received sequence) and **duplicates** (last-wins `insert()`; the official `valuelist.xml` sample contains a duplicate id). No mechanical uniqueness constraint protects any collection 0010 models — the only `xs:unique` in the schema set covers Category and MetadataAttribute, and the "each key component only once" rule for regions is prose. A "hybrid" policy (map where the schema guarantees uniqueness, `Vec` where it does not) therefore collapses to `Vec` everywhere within this crate's scope; the per-type calls already made (`ValueItem` D-0047, `Group` D-0049) anticipated this.

**Decision** (review decision, 2026-06-11): **every wire collection is stored as an ordered `Vec` in wire order**; lookup is a first-match Layer-2 view; duplicate-identity entries are preserved verbatim and flagged by catalogued lints. The sweep: `ItemScheme<I>.items` → `Vec<I>` (`insert()` → push; `get(id)` → first-match view; with the key-derivation invariant gone, `ItemScheme` becomes a pub-field carrier with derived `Deserialize` per ADR-0021's strict rule — the §3 key/id-desync rationale no longer applies); `LocalisedString` → an ordered list of `(language, text)` entries (`first()` = first in wire order — still deterministic; `get(lang)` = first match; the non-empty and non-blank-key invariants are unchanged); `AttributeList` content → a **single** `Vec` of a two-arm member enum (`Attribute` | `MetadataAttributeUsage`) because the wire is one repeated choice — two parallel `Vec`s would erase the interleaving — with `attributes()`/`usages()` filtered views; `MeasureList.measures` → `Vec<Measure>`; the `CubeRegion` and `DataKey` selection collections → `Vec`s of their node structs, whose ids move **into** the structs (`pub id: String`, structural-only, exactly as the map keys were); `DataStructureDefinition.groups` was already a `Vec` (D-0049). Dual representation (cached index views over the `Vec` store) is the sanctioned later evolution if profiling demands — additive and non-breaking, which is why `Vec` is the correct store side to fix now.

**Rationale**: The store rule stays exception-free (no per-collection judgment about whether order or duplicates "matter" — the demonstrated architectural flaw), the store-exactness property test becomes total, and reversibility is asymmetric (`Vec` → views is additive; map → `Vec` is a breaking change). Lookup cost is O(n) at SDMX metadata cardinalities (D-0006's own 10–5,000 analysis), and the genuinely hot per-observation paths belong to the parser/client crates, which build their own indexes.

**Consequences**: (1) D-0006 superseded (note added). (2) Design §2/§3/§5 swept: the `ItemScheme` framework narrative, `LocalisedString`, the D-0049 descriptors, and the D-0038/D-0039 selection collections. (3) Duplicate-id and duplicate-language lints join the catalogued surface (D-0031/ADR-0023). (4) Writers emit collections in stored order — wire-order determinism replaces sorted-order determinism.

---

### D-0052 — Attribute statedness stored: XSD defaults and fixed values are views, not data

> **Sweep gap closed 2026-06-11 by [D-0057](#d-0057)**: `TimeDimensionType.id` (`use="optional" fixed="TIME_PERIOD"`) belongs to this entry's fixed-attribute class and lands with D-0057's `ComponentMetadata` (statedness stored; `Some(v)` ≠ `"TIME_PERIOD"` rejected).
>
> **Additional four sweep gaps closed 2026-06-11**: the per-ref `optional` on `AttributeRelationship` dimension refs (`default="false"`, wholly unmodelled — [D-0058](#d-0058)); `TextType`'s `xml:lang` (`default="en"` — the `LocalisedString` key, [D-0059](#d-0059)); `TimePeriodRangeType.isInclusive` (`default="true"` → `Option<bool>` + `effective_is_inclusive()` view); and `Representation.max_occurs` (the sweep listed `min_occurs` only — `maxOccurs` defaults to `1` at the Basic/Measure positions, a position-aware effective view; see the D-0048 correction note).
>
> **Corrected 2026-06-11**: the sweep line below over-claims a `String` default for `EnumerationFormat.text_type` — `CodedTextFormatType` re-declares `textType` with **no** default (both versions; the restriction replaces the base declaration), so absent means *unrestricted* and no `effective_*()` default applies at the coded position. The `Option<DataType>` store is unaffected.

| **Area**     | Architecture |
| **Phase**    | Phase-1 |
| **Status**   | Active (sweep gap closed by [D-0057](#d-0057)) |
| **Keywords** | statedness, defaults, fixed, option, views, document-integrity, verbatim-store, spec-alignment |
| **Spec ref** | XSD attribute declarations with `default` or `fixed` across all in-scope schemas (see sweep list); XSD assessment (PSVI) semantics |
| **Source**   | [ADR-0023](adr/0023-two-layer-infoset-store-and-derived-views-architecture.md); [ADR-0024](adr/0024-byte-preserving-document-integrity-pathway.md) |
| **Related**  | [D-0010](#d-0010), [D-0022](#d-0022), [D-0025](#d-0025), [D-0026](#d-0026), [D-0028](#d-0028), [D-0030](#d-0030), [D-0032](#d-0032), [D-0038](#d-0038), [D-0039](#d-0039), [D-0045](#d-0045), [D-0046](#d-0046), [D-0048](#d-0048), [D-0049](#d-0049), [D-0051](#d-0051) |

**Observation**: A document that states a defaulted attribute and one that omits it are different documents — different bytes, different Infoset. XSD assessment fills in defaults and fixed values *post-validation* (the PSVI); that fill-in is an **interpretation of the document, not the document**. Several decisions stored the post-default form as a bare value (`bool`, `u32`, `DataType`, `Usage`, `Cascade`), erasing stated-vs-absent — structurally the same collapse D-0031 forbids, justified by "the schema's own equivalence". The exactness discussion rejected that justification: **XSD defaulting is a view over the data, not the data itself**, and the document-integrity pathway (ADR-0024) requires the model never to force a document change. The one prior `Option` forced by a flipped default (`isMultiLingual`, D-0046) was the general rule showing through a special case.

**Decision** (review ruling, 2026-06-11): every optional attribute with a schema `default` or `fixed` value stores **statedness**: `Option<T>` with `None` ⟺ absent on the wire; the applied default is a Layer-2 `effective_*()` view. For `fixed` attributes, a stated value differing from the fixed value is mechanically schema-invalid, so `new()`/`Deserialize` reject it (`Error::FixedAttributeMismatch`); statedness itself is stored. The sweep (each named entry amended): `is_partial_language` (D-0010) and `is_external_reference` (D-0030) on `MaintainableMetadata` → `Option<bool>`; `ItemScheme.is_partial` (D-0032) → `Option<bool>`; the region-level and selection-level `include`s (D-0026/D-0038/D-0039) → `Option<bool>` (default `true`); `Cascade` on every value type (D-0026/D-0040) → `Option<Cascade>` (default `false`); `Usage` on `Attribute`/`Measure` (D-0025) → `Option<Usage>` (default `optional`); `TextFormat.text_type` and `EnumerationFormat.text_type` (D-0028/D-0048) → `Option<DataType>` (default `String`; the time position's differing default `ObservationalTimePeriod` makes the effective view position-aware, supplied at the component level); `Representation.min_occurs` (D-0048) → `Option<u32>` (default 1); `evolving_structure` (D-0045) → `Option<bool>`; the `fixed="true"` includes on `DataKey`/`DataKeyValue` (D-0039) → `Option<bool>` with `Some(false)` rejected; the fixed descriptor ids (D-0049) → `Option<String>` with mismatch rejected; and `AgencyScheme`'s required `fixed="AGENCIES"` id — required, so no statedness, but the **mismatch check was missing**: `AgencyScheme::new()` becomes fallible, rejecting any other id (D-0023's "infallible" claim amended). `isMultiLingual` (D-0046) is already conformant and is now the general rule, not an exception.

**Rationale**: This is D-0031 applied to its own gap: the PSVI is a *view* by the architecture's own definition, and storing it in place of the document is a strict violation of ADR-0023. With statedness stored, writers reproduce stated attributes verbatim and omit absent ones — statedness round-trips, the canonical pathway's contract (ADR-0023) holds without a defaults carve-out, and the document-integrity pathway is never forced to diff.

**Consequences**: (1) `Error` gains `FixedAttributeMismatch`. (2) Each swept entry carries a one-line amendment note; D-0022's residual "semantic" clause is retired (note added there). (3) The `effective_*()` view family grows accordingly (catalogued; views are non-breaking to add or revise). (4) Constructor signatures change for the affected types (`Option` parameters); `AgencyScheme::new()` becomes fallible. (5) Design §5 swept throughout; ADR-0023 records the principle.

---

### D-0053 — Dataflow.dsd is Option by design (external-reference stubs)

| **Area**     | Dataflow |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | dataflow, dsd-reference, optionality, external-reference, spec-alignment |
| **Spec ref** | [SDMXStructureDataflow.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureDataflow.xsd#L36-L61) (`Structure` `minOccurs="0"`, line 47); [3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXStructureDataflow.xsd#L12-L32) (line 23, identical) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.7 |
| **Related**  | [D-0030](#d-0030), [D-0031](#d-0031) |

**Observation**: The `Structure` element (the dataflow's DSD reference) is `minOccurs="0"` in both versions. The spec's prose says the structure must be referenced "unless defined externally" — an external-reference stub (D-0030) may legitimately omit it. The model's `dsd: Option<DataStructureReference>` was correct but silent: no comment, no register record, unlike every comparable decision.

**Decision**: Record the `Option` as deliberate. `None` is a schema-valid wire state (typically an `isExternalReference=true` stub whose full definition lives elsewhere). The prose conditional ("must reference a DSD unless defined externally") is **not** a construction rejection: it is stated only in documentation, so a non-stub dataflow without a `Structure` still validates against the XSD — under ADR-0023 the coherence check ("`dsd: None` while the effective `is_external_reference()` is false is dubious") is a catalogued lint.

**Consequences**: (1) The design field gains its rationale comment. (2) The stub-coherence lint joins the catalogued surface.

---

### D-0054 — CodelistExtension modelled on Codelist; geo-codelist artefacts recorded out of scope

| **Area**     | Codelist |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | codelist-extension, prefix, member-selection, geo-codelist, scope-boundary, spec-alignment |
| **Spec ref** | [SDMXStructureCodelist.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureCodelist.xsd#L94-L118) (`CodelistExtensionType` — on `CodelistType` 0..unbounded; `CodeSelectionType`; `MemberValueType`; `GeographicCodelistType`/`GeoGridCodelistType`); [3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXStructureCodelist.xsd#L89-L113) (identical extension structure) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.5 |
| **Related**  | [D-0034](#d-0034), [D-0038](#d-0038), [D-0040](#d-0040), [D-0051](#d-0051), [D-0052](#d-0052) |

**Observation**: `CodelistType` carries `CodelistExtension` 0..unbounded in both versions — a codelist may be composed by extending others — and the official `codelist - extended.xml` sample exercises it. Verified structure (identical 3.0/3.1): a required `Codelist` reference, an **optional** choice of `InclusiveCodeSelection` | `ExclusiveCodeSelection` (each `MemberValue+`, where `MemberValueType` is a wildcardable string with an optional `cascadeValues` that has **no schema default**), and an optional `prefix` (`xs:string`, no default) — the prefix that the selection-level `removePrefix` flag (D-0038) refers to. Separately, `GeographicCodelistType`/`GeoGridCodelistType` are distinct artefact classes (with geo item types) the design has never claimed.

**Decision**: Model the extension (the standing full-model ruling): `Codelist` gains `extensions: Vec<CodelistExtension>` (0..unbounded; empty ⟺ absent). `CodelistExtension { codelist: CodelistReference, selection: Option<CodeSelection>, prefix: Option<String> }`; `CodeSelection = Inclusive(MemberValues) | Exclusive(MemberValues)`; `MemberValues` is the bespoke non-empty newtype (`MemberValue+` is mechanical; `Error::EmptyMemberValues`); `MemberValue { value: String, cascade: Option<Cascade> }` — the value stored verbatim (wildcard semantics are content, not grammar), and `cascade` an `Option` with **no** effective-view default, because the schema declares none (contrast D-0052's defaulted sites). **Geo boundary recorded as a cut** (the D-0034 pattern): `GeographicCodelist`/`GeoGridCodelist` are out of 0010 scope alongside the other unclaimed artefact classes (hierarchical codelists, categorisations, …); revisit if geospatial structures enter scope. The boundary is now explicit, not silent.

**Consequences**: (1) `Error` gains `EmptyMemberValues`. (2) `removePrefix`↔extension coupling closes: both halves are now modelled. (3) `ValueList` (D-0047) carries no extension element — nothing to mirror.

---

### D-0055 — Contact modelled on Agency

| **Area**     | Organisation |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | contact, agency, organisation, interleaving, spec-alignment |
| **Spec ref** | [SDMXStructureOrganisation.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureOrganisation.xsd#L327-L376) (`ContactType`; `OrganisationType.Contact` 0..unbounded, line 85); [3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXStructureOrganisation.xsd#L320-L369) (identical) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.5 |
| **Related**  | [D-0016](#d-0016), [D-0035](#d-0035), [D-0051](#d-0051) |

**Observation**: Every organisation carries `Contact` 0..unbounded (identical both versions); the modelled `Agency` silently dropped producer-supplied contact data — the defect class D-0035 fixed for `Link`. Verified `ContactType`: `Name*`, `Department*`, `Role*` (all localisable `TextType`, 0..unbounded), then **one repeated choice** (0..unbounded) of `Telephone`/`Fax`/`X400`/`URI`/`Email`.

**Decision**: Model it: `Agency` gains `contacts: Vec<Contact>` (0..unbounded; empty ⟺ absent). `Contact { names: Option<LocalisedString>, departments: Option<LocalisedString>, roles: Option<LocalisedString>, details: Vec<ContactDetail> }` where `ContactDetail = Telephone(String) | Fax(String) | X400(String) | Uri(String) | Email(String)` — **one interleaved Vec** because the wire is one repeated choice (the D-0051 `AttributeListMember` precedent: parallel per-kind Vecs would erase the interleaving). The localisable triple reuses `LocalisedString` exactly as artefact names do, inheriting whatever resolution the open `xs:language`-key thread lands. Contacts on the *other* organisation kinds (data/metadata providers, organisation units) ride those unmodelled schemes — out of 0010 scope, recorded here.

**Consequences**: (1) `Agency::new()` gains the parameter (invariant-free field; no new check). (2) `Contact`/`ContactDetail` are pub-field carriers (derived).

---

### D-0056 — effective_position pinned 1-based (derived fallback = list index + 1)

| **Area**     | Data structure |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | position, effective-view, convention, lint, spec-alignment |
| **Spec ref** | [SDMXStructureDataStructure.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureDataStructure.xsd#L344-L364) (`BaseDimensionType.position`, `xs:int`, optional, base unstated); official sample `ECB_EXR.xml` (`position="1"`…`"5"` for five dimensions in declaration order) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.6 |
| **Related**  | [D-0022](#d-0022), [D-0031](#d-0031) |

**Observation**: The XSD types `position` as `xs:int` and says nothing about its base; the official ECB_EXR sample states positions **1-based** in declaration order. The view's fallback returned the raw 0-based list index, so a stated-position DSD and its omitted-position equivalent produced views offset by one, and the planned position-consistency lint was unwritable without a pinned convention — under the current view the official sample's stated positions would all have been flagged inconsistent.

**Decision**: Pin the convention: `Dimension::effective_position(list_index)` returns the stated value when present, else **`list_index + 1`** (1-based, matching official stated-position usage). The store is untouched (`Option<i32>`, verbatim — D-0022 as re-homed under ADR-0023); only the *view's* derivation is pinned. The position-consistency lint is now writable: flag a stated position that differs from `list_index + 1`.

**Consequences**: (1) The view's doc comment carries the convention and its sample-based justification. (2) The catalogued lint gains its precise predicate.

---

### D-0057 — Component id statedness stored (ComponentMetadata); the trait id() is the effective view

> **Totality claim re-dated 2026-06-11**: two further Layer-1 holes found after this entry recorded consequence (3) — the per-ref `optional` attribute on `AttributeRelationship` dimension refs and the absent-`xml:lang` statedness on `LocalisedString`. Both are closed by [D-0058](#d-0058)/[D-0059](#d-0059); the claim holds again as of their landing.

| **Area**     | Data structure |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | component, id-inheritance, statedness, effective-view, time-dimension, fixed, trait-boundary, spec-alignment |
| **Spec ref** | [SDMXStructureBase.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureBase.xsd#L151-L168) (`ComponentBaseType.id`, `NCNameIDType`, `use="optional"`); [SDMXStructureDataStructure.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureDataStructure.xsd#L396-L413) (`TimeDimensionType.id`, `use="optional" fixed="TIME_PERIOD"`) |
| **Related**  | [D-0029](#d-0029), [D-0031](#d-0031), [D-0051](#d-0051), [D-0052](#d-0052) |

**Observation**: A component's `id` is `use="optional"` (both versions): when absent, the component's identity is inherited from its concept identity. The model required an id, so a schema-valid id-less component was unrepresentable. D-0052 now stores statedness even for attributes the schema itself fills, and a stated-vs-inherited id is more clearly document information than a defaulted boolean; and `TimeDimensionType.id` is an optional **fixed** attribute (`"TIME_PERIOD"`).

**Decision**: Build it. A components-only metadata leaf — `ComponentMetadata { id: Option<String>, uri, urn, annotations, links }` — whose `new()` NCName-validates the id **when stated** (`None` ⟺ inherited; loosening `IdentifiableMetadata.id` globally was rejected: every other identifiable artefact's id is mechanically required and the shared chokepoint must keep enforcing that). `Dimension`/`Attribute`/`Measure`/`TimeDimension` swap to it. **The trait is the domain boundary**: the components' `IdentifiableArtefact::id()` returns the *effective* identity — the stated id, else the concept reference's item id (`"TIME_PERIOD"` for the time slot) — because the inherited id *is* the component's identity in the spec's information model; the raw Layer-1 state stays reachable via `stated_id() -> Option<&str>` as the safety valve (D-0031 convention #3: expose both, the `position`/`effective_position` shape). `TimeDimension`: a stated id differing from `"TIME_PERIOD"` is mechanically schema-invalid → rejected (`FixedAttributeMismatch`, the D-0052 rule).

**Rationale**: The deferral's original framing ("derivable-optional canonicalisation, cf. position") was overturned for position itself and had become an inconsistency rather than a scoping choice. The D-0051 `Vec` store is **vindicated** here: under the old id-keyed maps, an optional id would have forced keying by the effective id — baking a Layer-2 view into the store's structure — or unrepresentability; with ordered `Vec`s the optional id is a pure field concern, and the lookup views (`AttributeList::get`, `MeasureList::get`) resolve by effective id exactly as the spec intends.

**Consequences**: (1) D-0025's deferral clause (consequence 2) is superseded (note added); the D-0023/D-0025 component NCName check becomes conditional-on-stated, living in `ComponentMetadata::new()`. (2) D-0052's sweep list gains its missed site (note added there). (3) With this, **no known schema-valid wire shape in 0010's scope is unrepresentable** — the Layer-1 claim is total.

---

### D-0058 — AttributeRelationship dimension refs carry the per-ref optional attribute (DimensionRef)

> **Carrier class amended 2026-07-08 by [D-0077](#d-0077)**: `DimensionRef` is no longer an invariant-free pub-field carrier — it crosses to an invariant-bearing type whose `new()` validates `id` as `NCNameIDType` (the `OptionalLocalDimensionReferenceType` base). The statedness treatment of `optional` and `DimensionRefs`' non-empty invariant stand.

| **Area**     | Data structure |
| **Phase**    | Phase-1 |
| **Status**   | Active (carrier class amended by [D-0077](#d-0077)) |
| **Keywords** | attribute-relationship, dimension-reference, optional, statedness, superset, spec-alignment |
| **Spec ref** | [SDMXStructureDataStructure.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureDataStructure.xsd#L192-L220) (`AttributeRelationshipType.Dimension` line 203; `OptionalLocalDimensionReferenceType` lines 222–228); [3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXStructureDataStructure.xsd#L180-L208) (lines 191/210–216, identical) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.6 |
| **Related**  | [D-0019](#d-0019), [D-0020](#d-0020), [D-0031](#d-0031), [D-0052](#d-0052), [D-0057](#d-0057) |

> **Accessor renamed 2026-06-29:** `effective_optional()` → `effective_is_optional()`, synchronising to the `effective_is_*` boolean effective-view naming convention; semantics unchanged. A name sync, not a new decision, so the reference below is updated in place.

**Observation**: Each `<Dimension>` ref inside `AttributeRelationshipType`'s Dimensions arm is typed `OptionalLocalDimensionReferenceType` in BOTH versions — an extension of `common:NCNameIDType` adding `optional: xs:boolean default="false"`. The model stored the refs as `DimensionRefs(Vec<String>)`, so `<Dimension optional="true">FREQ</Dimension>` — schema-valid wire — was unrepresentable. Doubly defective: a whole attribute dropped (the ADR-0008 #1 / Layer-1 superset hole), and a *defaulted* attribute, so also a missed D-0052 statedness site. It directly falsified D-0057's consequence (3) totality claim as recorded.

**Decision**: Model it. A per-ref struct `DimensionRef { id: String, optional: Option<bool> }` (named for the role; the spec type name is the unwieldy `OptionalLocalDimensionReferenceType`) — the id structural-only (D-0020), `optional` with statedness stored (D-0052: `None` ⟺ absent; `effective_is_optional()` = `false`). `DimensionRefs` becomes the bespoke non-empty newtype over `Vec<DimensionRef>` (its `EmptyAttributeDimensions` invariant and custom Deserialize unchanged); the `AttributeRelationship::dimensions()` forwarder takes `Vec<DimensionRef>`. An invariant-free pub-field carrier — position in the §7 taxonomy unchanged for every touched type.

**Rationale**: The standing full-model ruling leaves no room for a cut, and the statedness treatment is D-0052 applied mechanically. The recorded-cut alternative was rejected: it would have left the totality claim false for a trivially modellable attribute.

**Consequences**: (1) D-0052's sweep list gains the missed site (note added there). (2) D-0057's totality claim is re-dated (note added there): with this and D-0059, the no-unrepresentable-wire-shape claim holds again. (3) `MetadataAttributeUsage.relationship` shares the type, so usages gain the attribute for free.

---

### D-0059 — LocalisedString language key: statedness stored, validity is a view (parsable-within-spec)

| **Area**     | Localisation |
| **Phase**    | Phase-1 |
| **Status**   | Active (store-shape clause amended by [D-0066](#d-0066)) |
| **Keywords** | localisation, xml-lang, xs-language, statedness, defaults, parsable-within-spec, reject-line, lint, spec-alignment |
| **Spec ref** | [SDMXCommon.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXCommon.xsd#L118-L131) (`TextType` `xml:lang` `default="en"`, line 124); [3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXCommon.xsd#L114-L127) (line 120, identical); [xml.xsd](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/xml.xsd) (`xs:language` pattern) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.1 |
| **Related**  | [D-0011](#d-0011), [D-0016](#d-0016), [D-0031](#d-0031), [D-0051](#d-0051), [D-0052](#d-0052), [D-0055](#d-0055) |

> **Store-shape clause amended 2026-06-29 by [D-0066](#d-0066)**: the entry is now the named `LocalisedText { language, text }` struct, not the anonymous `(Option<String>, String)` tuple. The statedness (`language: Option<String>`), verbatim storage, effective-language view, and non-empty invariant all stand; only the element's representation is named. The accessors follow: `iter()` yields `&LocalisedText` and `as_slice()` is added.

**Observation**: Two axes of the same field, ruled together (review ruling, 2026-06-11). *Statedness*: `TextType` declares `xml:lang` with `default="en"` in both versions, and every localisable element in 0010 scope is `TextType` (Name/Description/AnnotationText/Contact triple) — so `<Name>Foo</Name>` and `<Name xml:lang="en">Foo</Name>` are distinct schema-valid documents that the `(String, String)` entry list collapsed (the parser would have had to bake the default in which would contradict the D-0052 PSVI-as-data move). *Validity* (the parked xs:language thread): `new()` rejected a blank stated key but stored off-pattern ones (`"e!"`, `"123"`) — an asymmetric middle ground, both shapes being mechanically schema-invalid against the `xs:language` pattern yet fully representable.

**Decision**: Both axes go verbatim. The store becomes `Vec<(Option<String>, String)>`: `None` ⟺ the attribute was absent; the `"en"` default is the per-entry `effective_lang()` view; `get(lang)` matches by effective language; `iter()` exposes the raw stated keys (the D-0031 expose-both convention). The blank-key rejection is **withdrawn**: a stated key — blank, off-pattern, or valid — is stored verbatim, and key well-formedness is a catalogued lint (design §5.11 #15). `MalformedLocalisation` is removed with its only producer (the §5.9 no-producerless-variants policy; rejoins on a MINOR bump if ever needed). The non-empty entry-list invariant is structural and **stands**. Governing principle, adopted as an ADR-0023 reject-line amendment: *mechanical schema invalidity is the ceiling of rejection, not a mandate* — data parsable within the constraints of the spec is not the library's to refuse; structural and identity/grammar-bearing invalidity (cardinalities, required members, the identifier tiers, the lexical newtypes, fixed-value mismatches) stays rejected, while a value-level lexeme in a content slot the store can hold verbatim may be ruled stored-plus-linted. Existing rejection sites are unchanged unless individually re-ruled.

**Rationale**: One field, one construction site, one coherent decision instead of two amendments. The statedness half is D-0052 applied to its missed attribute class; the validity half lands the parked thread's leaning — the key sits on the *content* side of the identity-vs-content fork (nothing structural depends on its grammar; worst case is can't-resolve-by-locale), so refusing it makes a call that belongs to the consuming application. Writers reproduce stated tags and omit absent ones, so key statedness round-trips need no language carve-out.

**Consequences**: (1) ADR-0023's reject-line is amended in place (ceiling-not-mandate; the value-level vs structural boundary); D-0031 carries a pointer note. (2) D-0016's key clauses are superseded (note added); its blank-value clause had already been withdrawn under D-0031. (3) `Error::MalformedLocalisation` removed; the duplicate-language lint becomes duplicate *effective* languages. (4) `Contact`'s localisable triple (D-0055) and `Annotation.texts` inherit the resolution wholesale.

---

### D-0060 — SdmxVersion ordering deferred past Phase 1 (raw Eq only, no Ord)

| **Area**     | Lexical types |
| **Phase**    | Phase-1 |
| **Status**   | Active (raw-Eq mechanism amended by [D-0070](#d-0070)) |
| **Keywords** | sdmx-version, ordering, ord, eq, semver, precedence, deferral, lexical |
| **Spec ref** | [SDMXCommonReferences.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXCommonReferences.xsd#L1608-L1613) (`VersionType`); [Semantic Versioning §11](https://semver.org/#spec-item-11) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.1 |
| **Related**  | [D-0024](#d-0024), [D-0027](#d-0027) |

> **Amended 2026-07-02 by [D-0070](#d-0070)**: `SdmxVersion`'s equality now derives over the parsed decomposition rather than the raw string; by the grammar's canonicity this is the same partition, so every distinction below (`1.0.0-rc` vs `1.0.0`, `3.1` vs `3.1.0`) is preserved. The ordering deferral itself stands unchanged.

**Observation**: `SdmxVersion` has raw-based equality (D-0027): two versions are equal iff their canonical strings match, so `1.0.0-rc` and `1.0.0` are correctly unequal. A SemVer §11 precedence `Ord` would order them (`1.0.0-rc` < `1.0.0`), but the unresolved legacy-vs-semantic equivalence (`3.1` vs `3.1.0`: equal under precedence, distinct under raw-`Eq`) means a precedence `Ord` bound to that `Eq` would violate the `Ord`/`Eq` consistency contract (`cmp == Equal` exactly when `==`). The earlier 0010 §5.1 pseudocode nonetheless showed `impl Ord`/`impl PartialOrd`, contradicting both the raw-`Eq` above and the shipped code.

**Decision**: Ordering is **deferred past Phase 1**. `SdmxVersion` implements raw-based `PartialEq`/`Eq` and `Display` only; no `Ord`/`PartialOrd` in Phase 1. When precedence comparison is needed it lands as an explicit convenience (a method or a comparison wrapper), so raw-`Eq` and SemVer precedence coexist without an `Ord`/`Eq` contract: distinct under equality, equal under precedence. 0010 §5.1 is corrected to drop the `Ord`/`PartialOrd` impls and record this.

**Rationale**: A type-level `Ord` would force a single answer to the legacy-equivalence question and bind it to `Eq`, the lossy collision D-0024/D-0027 avoid. Deferring keeps the raw store lossless and leaves the precedence semantics to be settled with samples when a consumer actually needs sorting; pre-1.0 an additive `Ord`-or-method is a clean MINOR bump.

**Consequences**: (1) 0010 §5.1 drops the `Ord`/`PartialOrd` pseudocode (corrected). (2) No code change: the shipped `SdmxVersion` already omits `Ord` and documents the deferral; its Design Notes now cite this entry. (3) Sorting/precedence consumers (for example "latest version") wait for the future convenience; none exists in Phase 1.

---

### D-0061 — MemberValue content held verbatim; WildcardedMemberValueType well-formedness is a Layer-2 lint

| **Area**     | Codelist |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | codelist-extension, member-value, wildcard, pattern, parsable-within-spec, lint, carrier, spec-alignment |
| **Spec ref** | [SDMXStructureCodelist.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureCodelist.xsd#L134-L141) (`WildcardedMemberValueType`, `xs:pattern` `[A-Za-z0-9_@$-%]+`); [3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXStructureCodelist.xsd#L129-L136) (identical) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.5, §5.11 |
| **Related**  | [D-0023](#d-0023), [D-0031](#d-0031), [D-0054](#d-0054), [D-0059](#d-0059) |

**Observation**: `MemberValueType` extends `WildcardedMemberValueType`, an `xs:string` restricted by the pattern `[A-Za-z0-9_@$-%]+`. The `+` makes the empty string mechanically schema-invalid, and the class constrains the characters (the `%` is the selection wildcard). Because `MemberValue` is a pub-field carrier (§5.5), a consumer or a lenient parse can nonetheless materialise `MemberValue { value: "".., .. }` or an off-pattern value.

**Decision**: `MemberValue` stays a pub-field carrier, stored verbatim; member-value well-formedness (non-empty, and the `WildcardedMemberValueType` character pattern) is a **catalogued Layer-2 lint** (design §5.11 #16), not a `new()` rejection. This applies the ADR-0023 ceiling-not-mandate principle exactly as [D-0059](#d-0059) did for the `xml:lang` key: a value-level lexeme in a content slot the store can hold verbatim is ruled stored-plus-linted, not refused.

**Rationale**: Three reasons converge. (1) Consistency: the member value is on the *content* side of the identity-vs-content fork (nothing structural depends on its grammar), the same place D-0059 put the `xml:lang` key and where `ValueItem.id` (lint #9) already sits. (2) Verbatim wire: the XSD pattern reads `$-%` as a range (`$`..`%`), excluding the literal `-`, yet `IDType` code ids admit `-`, so a strict pattern check would reject a member value referencing a valid code id containing `-`; holding verbatim avoids refusing parsable wire. (3) Phase scope: §5.11 lints are deliberately unbuilt in Phase 1, so the correct action is to catalogue, not to add a rejection.

**Consequences**: (1) Design §5.11 gains lint #16 (member-value well-formedness). (2) §5.5's "stored verbatim" gains a cite to this decision. (3) No code change and no new `Error` variant (the no-producerless-variants policy holds). (4) A future wire-conformant writer is responsible for emitting only pattern-valid member values; the store does not guarantee it.

---

### D-0062 — ItemSchemeArtefact trait deferred to its first generic consumer

| **Area**     | Item schemes |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | item-scheme, traits, generics, build-at-first-caller, deferral, api-surface, object-safety |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.5 |
| **Related**  | [D-0021](#d-0021), [D-0032](#d-0032), [D-0051](#d-0051) |

**Observation**: The scheme wrappers (`Codelist`/`ConceptScheme`/`AgencyScheme`) each forward `is_partial`/`get`/`iter`/`push` to their inner `ItemScheme<I>` via inherent methods. The artefact interfaces are otherwise exposed as traits (`IdentifiableArtefact` … `MaintainableArtefact`), so a shared `ItemSchemeArtefact` trait (an associated `Item` type plus `is_partial`/`get`/`iter_items`) would let generic code operate over any scheme without naming the wrapper. `isPartial` cannot ride the maintainable hierarchy — non-scheme maintainables (DSD/Dataflow/DataConstraint) lack it — so any such trait must be a dedicated one.

**Decision**: The shared scheme trait is **deferred to its first generic consumer**, not built in Phase 1. The wrappers keep their inherent forwarding methods. When a caller that iterates schemes generically appears (parsers/writers/applications, Phase 2+), introduce `ItemSchemeArtefact: MaintainableArtefact` with `type Item: SchemeItem` and `is_partial`/`get`/`iter_items` (the last via RPITIT, so the trait is not object-safe).

**Rationale**: The crate's build-at-first-caller discipline applies — the no-producerless-variants policy (D-0021), plus validators and `DataType::is_simple`/`is_time` deferred to their first callers — and the generic-over-schemes consumer lives above this crate, so the trait has no Phase-1 caller. An additive trait is a clean MINOR bump (phases.md), so deferral costs nothing now and avoids freezing speculative surface into the 0.1.0 API.

**Consequences**: (1) 0010 §5.5's "no shared item-scheme trait" note is reframed as deferred-not-rejected, recording this shape. (2) No code change; the wrappers' inherent methods stand. (3) When added, the trait is not object-safe (RPITIT `iter_items`), so it serves generic bounds, not `dyn` — unlike the existing artefact traits the tests use as trait objects.

---

### D-0063 — Internal serde projection: a lossless Rust round-trip, not the SDMX wire format

| **Area**     | Serialisation |
| **Phase**    | Phase-1 |
| **Status**   | Active (convergence clause amended by [D-0068](#d-0068)) |
| **Keywords** | serde, transparent, projection, wire-format, lossless, round-trip, phase-2-gate, foundation |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §6 |
| **Related**  | [D-0016](#d-0016), [D-0052](#d-0052), [D-0059](#d-0059) |

> **Amended 2026-07-01 by [D-0068](#d-0068)**: the convergence question this entry deferred is resolved. The internal projection never converges to the SDMX wire; the round-trip is verified through a non-wire format, and `serde_json` is removed from the crate. The projection-is-not-the-wire ruling below stands unchanged; only the "deferred to a Phase-2 gate" clause is superseded. Original body retained for provenance.

**Observation**: The domain types derive `serde::Serialize`/`Deserialize`, but it was unstated what that serialisation represents. Serde's default newtype-struct behaviour silently flattens to the inner value for JSON. This leaves the projection unpinned on the type and undefined for non-self-describing formats; consumers and the future parsers/writers have no stated contract to rely on.

**Decision**: The crate's derived serde is an **internal, lossless infoset round-trip** (the Rust composition, read and written directly), **not** the SDMX-ML/SDMX-JSON wire format. The within-field wrapper newtypes (`LocalisedString`, `FixedInclude`) carry `#[serde(transparent)]` to pin that projection explicitly and format-agnostically. The wire mapping is owned by `sdmx-parsers`/`sdmx-writers`. Whether the types' own serde should later converge to SDMX-JSON, or remain an internal projection, is **deferred** to a Phase-2 entry gate (ROADMAP Phase 2 -> Parsers).

**Rationale**: A lossless round-trip preserves the stored statedness exactly (the document-integrity contract, D-0052/D-0059). The wire shape is a separate concern owned by the serialisation crates. `#[serde(transparent)]` is JSON-output-neutral today while giving a defined projection for the non-JSON formats Phase 2 introduces. Converging the types' serde to the wire now would cross cut concerns and require reopening the Phase-1 foundation types, risking breaking changes.

**Consequences**: (1) 0010 §6 documents the projection model and cites this entry. (2) `LocalisedString` and `FixedInclude` carry `#[serde(transparent)]`, with no consumer-visible JSON change. (3) ROADMAP records the Phase-2 convergence entry gate, including the null-vs-omitted statedness sub-decision. (4) The convergence decision, when taken, reopens this entry.

---

### D-0064 — TimeRange carries TimeRangeValueType's wrapper-level validity attributes (validFrom/validTo)

| **Area**     | Constraints |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | constraint, time-range, validity, member-selection, statedness, spec-alignment, fidelity |
| **Spec ref** | [SDMXStructureConstraint.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureConstraint.xsd#L522-L552) (`TimeRangeValueType`, `TimePeriodRangeType`); [3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXStructureConstraint.xsd#L607-L637) (identical) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.8 |
| **Related**  | [D-0026](#d-0026), [D-0038](#d-0038), [D-0040](#d-0040), [D-0027](#d-0027), [D-0031](#d-0031) |

**Observation**: `TimeRangeValueType` is not a bare content choice. Beyond the before/after/between choice (each endpoint `TimePeriodRangeType`), the complexType declares two type-level attributes of its own, `validFrom` and `validTo` (`common:StandardTimePeriodType`, `use="optional"`, identical 3.0/3.1: 3.1 lines 550-551, 3.0 lines 635-636). The model drawn under [D-0026](#d-0026), inherited unchanged through [D-0038](#d-0038)/[D-0040](#d-0040), captured only the choice, leaving both attributes unmodelled, so a schema-valid `<TimeRange validFrom="…" validTo="…">` round-trips lossily, against [D-0031](#d-0031)/[D-0022](#d-0022). This is a latent miss in the member-selection validity cluster, not a Milestone-4 construct: it has existed since `TimeRange` was first drawn and surfaced only when M4 became the first build of the type. [D-0038](#d-0038)'s Observation records validity as a "three-level pattern (region: prohibited; dimension-selection: allowed; per-value: allowed)"; the `TimeRangeValueType` wrapper is a fourth occurrence that statement omits.

**Decision**: Remodel `TimeRange` from the bare `Before | After | Between` enum to a struct `TimeRange { kind: TimeRangeKind, valid_from: Option<SdmxTimePeriod>, valid_to: Option<SdmxTimePeriod> }`, where `TimeRangeKind` is the `Before(TimePeriodRange) | After(TimePeriodRange) | Between { start, end }` choice. The validity pair is `Option<SdmxTimePeriod>` ([D-0027](#d-0027)) because the attributes are `StandardTimePeriodType`, exactly `SdmxTimePeriod`'s domain, and so distinct from the endpoint content `TimePeriodRange.period`, which stays a `String` over the `ObservationalTimePeriodType` superset (unchanged). The attributes carry `use="optional"` with no schema default, so plain statedness (`None` ⟺ absent), no effective-view. The struct is a pub-field carrier with derived `Deserialize` (no between-field invariant; every field self-enforcing, §7); no new `Error` variant, the pair delegating to `SdmxTimePeriod::new()` (the existing `InvalidTimePeriod`).

**Rationale**: Model-not-cut, the governing precedent for the constraint cluster ([D-0026](#d-0026)/[D-0038](#d-0038)/[D-0040](#d-0040), ADR-0008): two optional schema-valid attributes are stored, not dropped. The fix is consistent rather than novel. Every sibling validity pair in §5.8 (`CubeKeyValue`, `SimpleComponentValue`, `CubeRegionKey`, `DataKey`) is already `Option<SdmxTimePeriod>` over the same `StandardTimePeriodType` attributes; `TimeRange`'s pair was the lone omission. `SdmxTimePeriod` already exists (Phase-1 foundation, [D-0027](#d-0027)), so there is no deferred dependency: this is not the `ObservationalTimePeriodType` lexical-typing work deferred for `.period`. Validity-of-selected-content **forks by choice arm**: per-value on the `Value+` arm ([D-0040](#d-0040)), per-wrapper on the `TimeRange` arm, the two realisations of one tier rather than a flat fourth level.

**Consequences**: (1) Corrects the [D-0038](#d-0038) record: its "three-level pattern" Observation is amended by blockquote to the content-validity-forks reading (original retained for provenance). (2) The `TimeRange` shape from [D-0026](#d-0026) is superseded by the `{ kind, valid_from, valid_to }` struct, with endpoints renamed `{ start, end }` to match the schema's `StartPeriod`/`EndPeriod`; `TimePeriodRange` is unchanged. (3) 0010 §5.8 is updated to the new shape, keeping design and register in sync. (4) No new `Error` variant and no custom `Deserialize`. (5) Implementation lands with the rest of `TimeRange` in Milestone 4; this entry corrects the design record now. (6) Validity is **per-construct**, not global. Only constraints and `TimeRangeValueType` use `StandardTimePeriodType`; structure maps use `xs:date`, and registry registration plus the dataset header use `xs:dateTime` (the header pair even named `validFromDate`/`validToDate`). When those constructs land (Phase 2+, other crates), each takes its own validity name and type: no shared `Validity` type, no blanket `valid_from`/`valid_to` rename.

---

### D-0065 — Equivalence and hash traits (Hash, Eq, PartialEq) derived uniformly where free; SdmxVersion hand-writes Hash

| **Area**     | Conventions |
| **Phase**    | Phase-1 |
| **Status**   | Active (SdmxVersion Hash carve-out amended by [D-0070](#d-0070)) |
| **Keywords** | hash, eq, partialeq, derive, map-key, hashmap, sdmx-version, uniformity, ergonomics, no-float |
| **Related**  | [D-0002](#d-0002), [D-0027](#d-0027), [D-0060](#d-0060) |

> **Amended 2026-07-02 by [D-0070](#d-0070)**: the `SdmxVersion` carve-out is retired. With the raw store dropped and equality structural, `SdmxVersion` derives the full `Hash`/`Eq`/`PartialEq` triple like every other float-free type, so the uniform baseline below now holds without exception. The `Hash`/`Eq` agreement is unchanged.

**Observation**: Equivalence and hashing traits have been applied as-needed, producing derive-set friction: a wrapped inner type may be `Hash` while its collection wrapper or parent aggregate is not. Because the crate models the strictly typed SDMX schema, almost every field is `String`, `bool`, an integer, a deterministic `enum`, `Vec`, `Option`, or `chrono::DateTime`, all of which are `Hash`; there are no floating-point values in the structural metadata chain. `PartialEq`/`Eq` are already derived crate-wide, so the live gap is `Hash`: today only the reference types ([D-0002](#d-0002)) derive it (an inline "natural map key" rationale), so equal value-model instances cannot serve as `HashMap`/`HashSet` keys, for no mechanical benefit. The one obstacle is `SdmxVersion`, whose `PartialEq`/`Eq` is hand-written over the raw canonical string ([D-0027](#d-0027)); it is embedded throughout the `VersionableMetadata` chain (hence every maintainable), so its `Hash` gates `Hash` on the whole versioned tree.

**Decision**: Adopt a uniform "where-free" baseline: every type that does not require floating-point representation carries `Hash`, `Eq`, and `PartialEq`. For the overwhelming majority (wrappers, float-free value enums, constraint leaf carriers, and top-level maintainables) this is the standard `#[derive(...)]`; in practice `PartialEq`/`Eq` are already universal, so `Hash` is the operative addition. `SdmxVersion` is the carve-out: it hand-writes `Hash` to hash only its raw string, agreeing with its lexical `Eq` (so `2.0` and `2.0.0` stay distinct and the `Hash`/`Eq` contract holds) rather than deriving over parsed components; establishing this unblocks `Hash` on the metadata tree. Forward rule: any new type derives the triple on creation unless it must store an `f32`/`f64`.

**Rationale**: `Hash`-where-free is zero-cost at runtime. Deriving `Hash` on large top-level aggregates (`Dataflow`, `Codelist`) that may never be map keys feels semantically impure, but the derived impl is dead code when unused and is stripped by the compiler/LLVM; only a microscopic macro-expansion compile-time tax remains, traded for absolute API consistency and frictionless downstream use. The `SdmxVersion` carve-out does not foreclose semantic version precedence (where `2.0` conceptually equals `2.0.0`): precedence is deliberately deferred to `Ord`/`PartialOrd` or a dedicated method ([D-0060](#d-0060)), decoupling mechanical map-key identity (`Hash`/`Eq`) from domain comparison logic.

**Consequences**: (1) The entire structural-metadata chain becomes trivially hashable, removing arbitrary roadblocks for downstream consumers. (2) The reference types' inline "natural map key" comment generalises to this entry and may cite it; the entry adds no forward reference to its own consumers. (3) Review rule: maintainers no longer litigate whether a type "semantically needs to be a key"; absent floats, it gets the triple. (4) `Borrow<str>` is unaffected: it stays conditional-only on the lexical newtypes (locking `Eq`/`Hash`/`Ord` could foreclose `SdmxVersion` precedence), so this entry adds `Hash` but does not adopt `Borrow`. (5) No behavioural change beyond the added traits; equality and serialisation semantics are unchanged.

---

### D-0066 — LocalisedString element named as LocalisedText (replacing the anonymous tuple)

| **Area**     | Localisation |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | localisation, localised-text, named-type, carrier, api-freeze, c-custom-type, ergonomics, serde |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.1 |
| **Related**  | [D-0016](#d-0016), [D-0051](#d-0051), [D-0059](#d-0059), [D-0063](#d-0063) |

**Observation**: [D-0059](#d-0059) stored the localisation entry as an anonymous `(Option<String>, String)` tuple. At the public API boundary the tuple's halves are positional and unnamed: a consumer reading `iter()` cannot tell the language tag from the text without the docs, and there is no element type to name in their own signatures.

**Decision**: Replace the anonymous tuple with a named `LocalisedText { language: Option<String>, text: String }`, an invariant-free public-field carrier mirroring `LocalisedString`'s derive set (plus derived `Deserialize`). `LocalisedString` wraps `Vec<LocalisedText>`; `iter()` yields `&LocalisedText`, `as_slice()` exposes the backing slice, and `new()`/`TryFrom<Vec<_>>`/the custom `Deserialize` take the named element. This revises only the element's representation; [D-0059](#d-0059)'s substance, language statedness (`Option`), verbatim storage, the effective-language view, and the non-empty invariant, stands. No `serde(from/into)` bridge: the array-to-object shape shift is the internal D-0063 projection, so the derives default.

**Rationale**: A named two-field struct names the language and text halves at the boundary the anonymous tuple obscured (the Rust API Guidelines' C-CUSTOM-TYPE), done pre-0.1.0 while the surface can still change without a release. The fields stay public because the carrier holds no invariant: the non-empty rule lives on `LocalisedString`, not its element. Construction takes `Vec<LocalisedText>` directly rather than re-blessing tuple input through a `From` conversion the freeze exists to retire.

**Consequences**: (1) [D-0059](#d-0059)'s store-shape clause is amended (note added there); its semantics are untouched. (2) `Contact`'s localisable triple ([D-0055](#d-0055)) and `Annotation.texts` reuse `LocalisedString` unchanged. (3) 0010 §5.1 snippet and value-model prose are updated to the named element, keeping design and register in sync. (4) The serde shape shifts from array-of-pairs to array-of-objects, immaterial pre-parsers and parked at the D-0063 Phase-2 gate. (5) `LocalisedText` is re-exported from the crate root; no new `Error` variant.

---

### D-0067 — ItemScheme kept a public, invariant-light generic carrier

| **Area**     | Item schemes |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | item-scheme, visibility, public-api, generics, carrier, wrapper-invariants, extensibility, sealing |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §3 |
| **Related**  | [D-0032](#d-0032), [D-0051](#d-0051), [D-0062](#d-0062) |

**Observation**: `ItemScheme<I>` is the generic carrier the scheme wrappers (`Codelist`/`ConceptScheme`/`AgencyScheme`) compose. It is a transparent pub-field carrier (ADR-0021) with an infallible `new()` plus `push`/`get`/`iter`, and carries no construction invariant of its own; the scheme-level invariants — the `NCNameIDType` scheme id (`Codelist`/`ConceptScheme`) and `AgencyScheme`'s fixed `AGENCIES` id — live on the concrete wrappers' fallible `new()`. A public-surface review raised whether the carrier should be made private or sealed.

**Decision**: Keep `ItemScheme<I>` public: a deliberately public, generic, invariant-light carrier, with the concrete wrappers owning the construction invariants. The rationale is that it lets downstream code process any scheme generically (over `I: SchemeItem`) without exposing a validation bypass, because the carrier holds no invariant for public exposure to bypass.

**Rationale**: Sealing or privatising the carrier would remove the generic, extensible processing the open-`SchemeItem` design (§3) presupposes, for no safety gain: there is no invariant on `ItemScheme` for a caller to break, and `I: IdentifiableArtefact` already supplies the only thing it needs. This mirrors §3's seal-only-when-openness-would-break-an-invariant policy (the `SdmxSerialize` contrast). The wrappers remain the validation boundary.

**Consequences**: (1) `ItemScheme<I>` stays `pub`; no code change. (2) 0010 §3 gains a sentence recording the carrier-versus-wrapper-invariants split. (3) Future generic scheme processing (e.g. the deferred `ItemSchemeArtefact`, [D-0062](#d-0062)) builds on the public carrier; sealing is off the table unless an invariant is later added to the carrier itself.

---

### D-0068 — Internal serde projection never converges to the SDMX wire; round-trip verified through a non-wire format

| **Area**     | Serialisation |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | serde, projection, wire-format, convergence, non-wire, round-trip, no_std, public-api |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §6 |
| **Related**  | [D-0063](#d-0063), [D-0031](#d-0031), [D-0066](#d-0066) |

**Observation**: [D-0063](#d-0063) fixed the derived serde as an internal lossless projection and deferred to a Phase-2 entry gate whether it should later converge to SDMX-JSON. Two facts settle that early. First, the projection is exercised only in tests, and it was exercised through `serde_json`, a JSON wire library in the infoset crate's dev-dependencies, which conflates the internal projection with a wire format `sdmx-parsers`/`sdmx-writers` own. Second, the custom `Deserialize` impls are format-neutral: each delegates to a standard-library type then routes through `new()`, with no `deserialize_any`, no untagged or internally-tagged enums, and no hand-rolled visitors, so the JSON choice is incidental, not structural.

**Decision**: The internal serde projection **never converges** to the SDMX wire format; convergence is resolved, not deferred. The round-trip losslessness property is verified by round-tripping through a **non-wire** serialisation format, and `serde_json` is removed from `sdmx-types` entirely, so the crate carries no wire-format dependency. serde stays a hard, public dependency, so the projection is a **public API surface whose shape is stable by decision**, acceptable because it never converges and so never breaks a consumer that serialises the model.

**Rationale**: Resolving convergence to "never" removes the only reason to hedge the public projection shape, a future wire-convergence that could change it and break serialising consumers, which in turn makes a serde feature-gate unnecessary. Verifying losslessness through a non-wire format makes "parsers own the wire" a property of the dependency graph rather than a doc comment: nothing in `sdmx-types` can be mistaken for owning the wire because no wire-format library is present. Converging the projection to SDMX-JSON instead would reopen the Phase-1 foundation types under the no-breaking-changes-without-a-MINOR-bump rule and couple the types to one wire format over the others.

**Consequences**: (1) [D-0063](#d-0063)'s deferred convergence clause is resolved; its body is amended by blockquote (retained for provenance). (2) The ROADMAP Phase-2 "serde wire-shape convergence" entry gate is retired; its live wire sub-questions (null-vs-omitted statedness, `LocalisedString` wire shape, enum representations) relocate to `sdmx-parsers`/`sdmx-writers`, where the wire lives. (3) 0010 §6 is updated to record convergence as resolved. (4) `serde_json` is removed from `sdmx-types`; the round-trip and construction-rejection tests are re-expressed through a non-wire format and the validated constructors, leaving no wire library in the crate (implementation tracked as a separate `test`-scoped follow-up). (5) Feature-gating serde is declined: with convergence resolved the projection shape is stable, so the gate it would hedge is unnecessary.

---

### D-0069 — Reference, version, and time-period grammars are model surface gating the 0.1.0 publish; the wire mapping stays with the parser/writer

| **Area**     | Architecture |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | reference-types, version, time-period, urn, wildcard, lexical-grammar, public-api |
| **Spec ref** | [SDMXCommonReferences.xsd 3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXCommonReferences.xsd#L1512-L1548) (`VersionReferenceType`, `SemanticVersionReferenceType`, `WildcardVersionType`), [L1606-L1629](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXCommonReferences.xsd#L1606-L1629) (`VersionType`), [L136-L173](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXCommonReferences.xsd#L136-L173) (URN version parts); [SDMXCommon.xsd 3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXCommon.xsd#L482-L488) (`ObservationalTimePeriodType`); [SDMX 3.0 Section 5, Registry Specification](https://sdmx.org/wp-content/uploads/SDMX_3-0-0_SECTION_5_FINAL-1_0.pdf) (URN macro grammar); [SDMX 3.0 Section 6, Technical Notes](https://sdmx.org/wp-content/uploads/SDMX_3-0-0_SECTION_6_FINAL-1_0.pdf) §14.2/§14.3 (semantic-version specification and BNF) |
| **Related**  | [D-0002](#d-0002), [D-0024](#d-0024), [D-0027](#d-0027), [D-0060](#d-0060), [D-0064](#d-0064), [D-0068](#d-0068) |

**Observation**: The roadmap filed the reference-types/URN pass as a Phase-2 parser entry gate. The schema locates it in the domain model: the spec separates the declaration version grammar (`VersionType`, exact) from the reference version grammar (`VersionReferenceType` adds single-segment `+` wildcarding; `WildcardVersionType` adds the bare `*`), refines a URN version part per reference context, and types `TimePeriodRange.period` as the `ObservationalTimePeriodType` union (`StandardTimePeriodType ∪ TimeRangeType`). The model currently holds plain `String` placeholders in exactly these positions: reference `version` fields, un-contracted reference URN text, and `TimePeriodRange.period`. `0.1.0` publishes the public API; a placeholder shape cannot be tightened afterwards without a breaking change.

**Decision**: The reference, version, and time-period grammar model is completed in the domain model before the `0.1.0` publish and gates it. The types carry the spec grammars (exact declaration version, wildcard version references, observational time periods, per-shape URN contract); only the wire mapping stays with the parser/writer per [D-0068](#d-0068).

**Rationale**: Grammar defines the value space of public fields, which is API shape; wire mapping defines how values render in messages, which [D-0068](#d-0068) already placed outside the model. Publishing placeholders inverts the cost of completing the grammar: an internal completion becomes a consumer-visible break.

**Consequences**: (1) Follow-up model passes, each register-first: the version model (`SdmxVersion` updated to the canonical raw-free form plus a `VersionRef` reference type; will amend [D-0027](#d-0027)'s lossless-raw clause, with [D-0060](#d-0060)'s ordering deferral standing), the observational time-period union type (intersects [D-0064](#d-0064)), and the reference URN `Display`/`FromStr` contract with named errors (will amend [D-0002](#d-0002); a reference is wire text that owes ADR-0024 a verbatim round-trip). (2) Format/parse round-trip property tests fold into the existing property-based-testing roadmap item. (3) `VersionQueryType` (`1.*`, `1.0.*`) is registry-query grammar and stays out of structural references; wildcard resolution to a concrete latest version needs a registry catalogue and stays out of the types.

---

### D-0070 — SdmxVersion drops the stored raw: the canonical VersionType grammar reconstructs the lexeme

| **Area**     | Lexical types |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | sdmx-version, canonical, raw-free, bijection, statedness, round-trip, hash, eq |
| **Spec ref** | [SDMXCommonReferences.xsd 3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXCommonReferences.xsd#L1606-L1629) + [3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXCommonReferences.xsd#L1608-L1613) (`VersionType`, `SemanticVersionNumberType`, `LegacyVersionNumberType`) |
| **Related**  | [D-0024](#d-0024), [D-0027](#d-0027), [D-0060](#d-0060), [D-0065](#d-0065), [D-0069](#d-0069) |

**Observation**: Every numeric component of `VersionType` is `0|[1-9]\d*` (no leading zeros, no sign) and each prerelease extension has exactly one spelling, so the grammar is canonical: one lexeme per value, and format-then-parse is a bijection. The raw stored by [D-0027](#d-0027) is therefore a redundant copy of the decomposition, except where it silently carried statedness the decomposition dropped: `minor: u32` folded the bare-major legacy form (`1`) into `1.0`, and only the raw kept them distinct. `xs:decimal`/`xs:integer` differ: `1.0` vs `1.00` and `7` vs `007` are distinct lexemes of equal value, so their raw is load-bearing.

**Decision**: `SdmxVersion` drops the stored raw and becomes statedness-preserving: `{ major: u32, minor: Option<u32>, patch: Option<u32>, extension: Option<String> }`, with `minor: None` encoding the bare-major legacy form. `Display` reconstructs the lexeme; `PartialEq`/`Eq`/`Hash` derive structurally (by the bijection, the same partition as comparing lexemes); `as_str()`/`AsRef<str>` are removed (no stored lexeme to borrow); serde serialises through `Display`, keeping the single-canonical-string projection shape. [D-0027](#d-0027)'s lossless-raw rule narrows to non-canonical grammars.

**Rationale**: A redundant store is a second source of truth that can only agree or drift, and it cost three hand-written impls (`PartialEq`/`Eq`/`Hash`) plus their contract apparatus, all replaced by derives. The masked statedness was the sharper defect: the raw made `1` and `1.0` compare unequal while the accessors reported them identical (`minor()` returned `0` for both); making `minor` optional puts the distinction where consumers actually read it.

**Consequences**: (1) Amends [D-0027](#d-0027): the `SdmxVersion` raw clause; `SdmxDecimal`/`SdmxInteger` stand. (2) Amends [D-0060](#d-0060): equality is structural rather than raw-based, the same partition; the ordering deferral stands. (3) Amends [D-0065](#d-0065): the `SdmxVersion` hand-written-`Hash` carve-out retires; the uniform derive baseline holds without exception. (4) Public API: `minor()` returns `Option<u32>`; `as_str()`/`AsRef<str>` are gone, rendering goes through `Display`. (5) Both round-trip directions (`x.to_string().parse() == Ok(x)` and `parse(s).to_string() == s`) are asserted in unit tests and fold into the property-based-testing roadmap item. (6) Design 0010 §5.1 reconciliation rides the series close-out.

---

### D-0071 — VersionRef models the version reference grammar; one + wildcard enforced across editions

| **Area**     | Lexical types |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | version-ref, wildcard, reference-types, lexical-grammar, canonical, spec-alignment |
| **Spec ref** | [SDMXCommonReferences.xsd 3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXCommonReferences.xsd#L1512-L1548) + [3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXCommonReferences.xsd#L1514-L1550) (`VersionReferenceType`, `SemanticVersionReferenceType`, `WildcardVersionType`, `WildcardType`) |
| **Related**  | [D-0002](#d-0002), [D-0021](#d-0021), [D-0046](#d-0046), [D-0060](#d-0060), [D-0069](#d-0069), [D-0070](#d-0070) |

**Observation**: A reference versions its target through a wider grammar than a declaration: `VersionReferenceType` adds `SemanticVersionReferenceType`'s single-`+` forms ("2+.3.1 means the currently latest available version >= 2.3.1", per the schema documentation), and `WildcardVersionType` adds the bare `*`. The `+` forms require the full semantic triple and exclude the prerelease extension ("2.3+.1-draft is not permissible", both editions). The editions diverge mechanically: 3.1's three patterns each admit `+` on exactly one component, while 3.0's third pattern also matches a double wildcard (`1+.2.3+`) that the same type's own documentation forbids.

**Decision**: A new exhaustive enum `VersionRef { Exact(SdmxVersion), Latest { major, minor, patch, at: WildcardPosition }, Any }` models `WildcardVersionType`, raw-free with a reconstructing `Display` (the [D-0070](#d-0070) canonicity fork). Exactly one wildcard is enforced across both editions, and the full-triple and no-extension requirements are structural: `Latest` carries three mandatory `u32` components and no extension slot, so the illegal combinations are unrepresentable. `Error` gains `InvalidVersionReference`; serde serialises through `Display`, keeping the single-canonical-string projection shape.

**Rationale**: Enforcing one wildcard deviates from [D-0046](#d-0046)'s carry-the-superset rule deliberately: the 3.0 double-wildcard match is regex slack, contradicted by the same type's documentation in both editions and by 3.1's tightened patterns of the identical documented contract, so carrying it would manufacture a value space no edition documents. Both enums are exhaustive rather than `#[non_exhaustive]` ([D-0021](#d-0021)): the union is grammar-closed.

**Consequences**: (1) The reference structs adopt `VersionRef` in the URN-contract pass, which also settles which reference contexts admit `*` versus only `+` ([D-0069](#d-0069)). (2) Wildcard resolution to a concrete version and the `VersionQueryType` query grammar stay out, per [D-0069](#d-0069). (3) The declaration/reference split is structural: `SdmxVersion` cannot hold a wildcard, and `VersionRef` is the only wildcard carrier. (4) Round-trip tests assert both directions and fold into the property-based-testing roadmap item.

---

### D-0072 — ObservationalTimePeriod union carries TimePeriodRange.period; SdmxTimeRange models the TimeRangeType lexeme

| **Area**     | Lexical types |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | time-period, time-range, observational, union, lexical-grammar, constraints, spec-alignment |
| **Spec ref** | [SDMXCommon.xsd 3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXCommon.xsd#L482-L488) + [3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXCommon.xsd#L492-L498) (`ObservationalTimePeriodType`); [3.0 L600-L672](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXCommon.xsd#L600-L672) + [3.1 L610-L682](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXCommon.xsd#L610-L682) (`TimeRangeType` and its restriction chain) |
| **Related**  | [D-0027](#d-0027), [D-0064](#d-0064), [D-0069](#d-0069), [D-0070](#d-0070) |

**Observation**: `TimePeriodRange.period` is typed `ObservationalTimePeriodType`, the union `StandardTimePeriodType ∪ TimeRangeType`. A time range is `start/duration` (a full `xs:date` or `xs:dateTime` with optional timezone, then an `xs:duration`) validated by a six-level restriction chain (base shape, month-day validity, leap years, time bounds, timezone bounds, duration shape). The model held `period` as a raw `String`: `SdmxTimePeriod` covers only the Standard member and would reject schema-valid time-range lexemes, and no type carried the range grammar.

**Decision**: Two additions. `SdmxTimeRange` is a lossless lexical newtype for `TimeRangeType` (raw stored: date and duration lexemes are not canonical, the [D-0070](#d-0070) fork; `Sdmx` prefix because bare `TimeRange` collides with the constraint selection type, [D-0027](#d-0027) naming rule), exposing `start()`/`duration()` slices. `ObservationalTimePeriod` is the exhaustive union `{ Standard(SdmxTimePeriod), Range(SdmxTimeRange) }`; the member grammars are disjoint (only a range contains `/`), so classification is unambiguous. `TimePeriodRange.period` adopts the union. `Error` gains `InvalidTimeRange` and `InvalidObservationalTimePeriod`.

**Rationale**: A union of the member newtypes rather than a widened `SdmxTimePeriod`: the Standard-only positions ([D-0064](#d-0064)'s `valid_from`/`valid_to` and the §5.8 validity pairs) must keep rejecting time-range lexemes, so the widening has to be a new type. The range's date half is validated by the shared Gregorian/date-time classifier, the same strictness the crate already applies to `xs:date`/`xs:dateTime` (the chain's month-length and leap-year patterns are not separately re-implemented); the duration is validated to the chain's ordered-component grammar.

**Consequences**: (1) `TimePeriodRange.period` tightens from `String` to `ObservationalTimePeriod`; its derived `Deserialize` now validates the union on the wire path. (2) The lexical-typing work [D-0064](#d-0064) recorded as deferred for `.period` is delivered; its body stands unchanged. (3) Round-trip tests fold into the property-based-testing roadmap item.

---

### D-0073 — Reference types own their class URN contract; versions typed VersionRef

| **Area**     | Reference types |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | reference-types, urn, display, fromstr, version-ref, wildcard, item-in-scheme, spec-alignment |
| **Spec ref** | [SDMXCommonReferences.xsd 3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXCommonReferences.xsd#L14-L204) (URN macro part chain) and the per-class reference simpleTypes throughout; [SDMXMetadataGeneric.xsd 3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXMetadataGeneric.xsd#L47) + [SDMXStructureProvisionAgreement.xsd 3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXStructureProvisionAgreement.xsd#L85) (the only `WildcardUrnType` consumers); [SDMX 3.0 Section 5, Registry Specification](https://sdmx.org/wp-content/uploads/SDMX_3-0-0_SECTION_5_FINAL-1_0.pdf) (URN macro grammar) |
| **Related**  | [D-0002](#d-0002), [D-0020](#d-0020), [D-0034](#d-0034), [D-0068](#d-0068), [D-0069](#d-0069), [D-0071](#d-0071) |

**Observation**: On the wire, every structural reference is URN text of a per-class simpleType, and all seven modelled classes descend from the URN *reference* chain (`UrnReferenceVersionPart`): the version part admits the `+` wildcard forms, and its character class (`[0-9A-Za-z\-\.\+]`) excludes the bare `*`. The `WildcardUrnType` family that admits `*` (and wildcard agency/id parts) is consumed only by metadata Target elements, which are not modelled. The URN mandates the version part on every class, including item-in-scheme (`agency:scheme_id(version).item`), whose scheme version the model had dropped, so the item references could not render a legal URN; agencies and item tails may be dot-nested.

**Decision**: Each reference struct owns its URN contract: `Display` renders the full class URN (`urn:sdmx:org.sdmx.infomodel.<package>.<Class>=...`) and `FromStr` parses exactly that class into the decomposed fields, with `Error::InvalidReferenceUrn { urn, class }` naming the expected class. All seven `version` fields are typed `VersionRef` ([D-0071](#d-0071)), and `ConceptReference`/`DataProviderReference` gain the mandated scheme version. `VersionRef::Any` is grammar-unparseable in every reference URN and rejected by `FromStr`; it remains carrier-representable like any other unvalidated field, catalogued as a Layer-2 lint. Item tails are held verbatim (nested paths are wire-legal). The serde impls stay field-wise derived: the internal projection is not the wire ([D-0068](#d-0068)).

**Rationale**: The parse path is where the grammar belongs: the Phase-2 parsers split wire references through `FromStr` ([D-0069](#d-0069)), while construction stays invariant-free because identifiers are validated at declaration, not reference ([D-0020](#d-0020)); rendering a wire-conformant URN from hand-built fields is the writer's obligation, like every other carrier field. The version is `VersionRef` rather than `SdmxVersion` because the chain says so: every reference class admits the `+` forms a declaration version cannot carry.

**Consequences**: (1) Resolves [D-0027](#d-0027) consequence (3): the references adopt `VersionRef`, not `SdmxVersion` (amended in place). (2) Extends [D-0002](#d-0002) without amendment: each distinct type now also owns its class URN; [D-0069](#d-0069)'s anticipated D-0002 amendment resolves as this extension. (3) Public API: the five triple `version` fields change type, and the two item-in-scheme structs gain a `version` field. (4) Two Layer-2 lints are catalogued in 0010 §5.11: a reference whose fields cannot render a wire-conformant URN (`VersionRef::Any` included) fails the writer's conformance check; and a dot-nested item tail on a flat item class (`Concept`, `DataProvider`), which the XSD patterns mechanically admit but the Registry Specification's URN prose forbids, is held verbatim and flagged. (5) The `*`-admitting contexts (metadata targets) take their own modelling when they arrive, not this shape. (6) Round-trip tests fold into the property-based-testing roadmap item.

---

### D-0074 — `PartialEq<str>` confined to the lexeme-storing lexical types

| **Area**     | Lexical types |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | partial-eq, string-comparison, operator-semantics, lexical, raw-backed, raw-free, api-surface, deferral |
| **Related**  | [D-0027](#d-0027), [D-0060](#d-0060), [D-0070](#d-0070), [D-0071](#d-0071), [D-0072](#d-0072) |

**Observation**: practices.md's trait-surface table lists `PartialEq<str>`/`PartialEq<&str>` as "intended" for the lexical newtypes without qualification, a row predating the split of the family by storage model. On the lexeme-storing types the stored text is the datum ([D-0027](#d-0027)), so comparison to a literal has exactly one reading: string identity. That covers the raw-backed four (`SdmxDecimal`, `SdmxInteger`, `SdmxTimePeriod`, `SdmxTimeRange`) and equally the `ObservationalTimePeriod` union ([D-0072](#d-0072)), whose members both store their lexeme and whose grammars are disjoint, so string identity coincides with structural equality. On the raw-free grammar types (`SdmxVersion` per [D-0070](#d-0070), `VersionRef` per [D-0071](#d-0071)) the lexeme is reconstructed, and `version == "1.0.0"` is guessable two ways — lexeme identity or version equivalence — the contested-operator shape that deferred `Ord` ([D-0060](#d-0060)); ecosystem precedent agrees (`semver::Version` carries no `PartialEq<str>`).

**Decision**: The operator follows the storage model: `PartialEq<str>`/`PartialEq<&str>` are implemented on the lexeme-storing types — the raw-backed four and the `ObservationalTimePeriod` union — as string identity with the stored lexeme, and the raw-free grammar types take no string-comparison operator.

**Rationale**: A comparison operator must have one obvious reading at the call site. Where the lexeme is reconstructed, the operator would silently pick one of two readings users legitimately hold — the hazard [D-0060](#d-0060) resolved by deferral — while declining costs nothing, because the operator is additive whenever its semantics are settled.

**Consequences**: (1) practices.md's trait-surface row is recut from unqualified "intended" to the lexeme-storing types, citing this entry. (2) Parse-based equality on the raw-free types is rejected outright, not merely deferred: under the bijective canonical grammars it duplicates structural equality wherever the literal is grammar-valid, and any divergence would bless non-stated forms against the statedness principle. (3) Canonical (formats-to-equal) semantics are pre-agreed as the meaning if ergonomic pressure ever justifies adopting the operator on a raw-free type — an additive MINOR change.

---

### D-0075 — Schema-unbounded integers take u32 width; the bound is a recorded deviation

| **Area**     | Conventions |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | integer-width, u32, nonzero, positive-integer, magnitude, deviation, representability |
| **Related**  | [D-0027](#d-0027), [D-0070](#d-0070), [D-0076](#d-0076) |

**Observation**: Several schema slots are unbounded integers — the `xs:positiveInteger` facet counts, the `xs:nonNegativeInteger` `minOccurs`, `MaxOccursNumberType`, and the `VersionType` numeric components — and the crate models them at `u32` width, so a schema-valid `minLength="5000000000"` or a beyond-`u32` version component is unrepresentable. [D-0070](#d-0070)'s design note argues the version components' *signedness*, not their magnitude, and no entry records the width choice, in contrast to the lexeme-storing types ([D-0027](#d-0027)) where the text is the datum and is held losslessly.

**Decision**: Schema-unbounded integers whose value is a count or version component are modelled at `u32` width (`u32`, or `NonZeroU32` where the schema floors the value at 1), and that width is a deliberate, documented deviation from the schema's unbounded value space. Where the value itself is the datum, the lexeme newtypes (`SdmxInteger`, `SdmxDecimal`) hold the text losslessly and no width applies.

**Rationale**: A count beyond `u32` is schema-legal but semantically absurd (a four-billion-character minimum length), so lexeme-fidelity machinery would buy nothing there; the fork that matters is datum-versus-bound, and the datum side is already lossless. Recording the width once gives every future integer slot a citable rule and stops the deviation propagating unrecorded.

**Consequences**: (1) The existing width choices are instances of this policy — the `SdmxVersion` components (whose rustdoc already documents the rejection), `Representation.min_occurs`, and `MaxOccurs` — so they now have a citable rule and no change follows for them. (2) The format facets adopt the width through [D-0076](#d-0076), taking `NonZeroU32` where the schema floor is 1. (3) A future slot whose integer is genuinely data-carrying must not take an integer width; it takes a lexeme newtype per [D-0027](#d-0027).

---

### D-0076 — Format-facet validity moves into the field types: SdmxDuration and NonZeroU32

| **Area**     | Data structure |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | facets, duration, textformat, nonzero, positive-integer, validation, representation |
| **Spec ref** | [SDMXStructureBase.xsd 3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXStructureBase.xsd#L274-L309) (`timeInterval` `xs:duration`; `minLength`/`maxLength`/`decimals` `xs:positiveInteger`); [SDMXCommon.xsd 3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXCommon.xsd#L697-L711) (`OccurenceType`, `MaxOccursNumberType` `minInclusive 1`); 3.1 identical |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.6.1 |
| **Related**  | [D-0027](#d-0027), [D-0028](#d-0028), [D-0048](#d-0048), [D-0075](#d-0075) |

**Observation**: The construction contract holds that input an XSD validator could itself reject is a construction error, and the format facets escaped it in two ways: `time_interval` held any `String` (the [D-0028](#d-0028) cut, flagged as an uncommon facet), and the `xs:positiveInteger` facets (`minLength`, `maxLength`, `decimals`) plus `MaxOccurs::Count` admitted a stated zero (`MaxOccursNumberType` is `minInclusive 1`; the zero admission was never recorded as a cut). The facet carriers are pub-field types with no constructor, so constructor-side checks are not available there.

**Decision**: Validity moves into the field types. `time_interval` takes `SdmxDuration`, a lexeme-storing newtype over the full signed `xs:duration` grammar (the `TimeRangeType` chain's ordered-component scanner plus the leading `-` that plain `xs:duration` admits); the `xs:positiveInteger` facets and `MaxOccurs::Count` take `NonZeroU32` at the [D-0075](#d-0075) width, whose serde impl rejects a stated zero on the wire path. `min_occurs` stays `u32`: `xs:nonNegativeInteger` admits zero, so tightening it would reject schema-valid input.

**Rationale**: Within-field grammar belongs to the field's type (§7): a nested validated type enforces the wire path through a derived carrier with no constructor, the same shape as `SdmxTimePeriod` inside the constraint carriers; `NonZeroU32` proivdes the schema floor with a `core` type and no new API surface.

**Consequences**: (1) `Error::InvalidDuration` joins the lexical family's variants. (2) `SdmxDuration` and `SdmxTimeRange` keep distinct validators: the range half is unsigned by the chain's pattern, plain `xs:duration` is signed; the shared scanner is the reuse point, not the grammar.

---

### D-0077 — Local reference identifiers validate their lexical tier at construction; D-0020 narrows to referential integrity

| **Area**     | Identifiers |
| **Phase**    | Phase-1 |
| **Status**   | Active |
| **Keywords** | identifiers, references, validation, ncname, idtype, construction-contract, representability, superset, spec-alignment |
| **Spec ref** | [SDMXStructureDataStructure.xsd 3.0](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXStructureDataStructure.xsd#L180-L275) (`AttributeRelationshipType.Group` `IDType` :197; `OptionalLocalDimensionReferenceType` base `NCNameIDType` :212; `MeasureRelationshipType.Measure` `NCNameIDType` :223; `MetadataAttributeReference` `NCNameIDType` :261; `GroupDimensionType.DimensionReference` `NCNameIDType` :463), [3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureDataStructure.xsd#L192-L287) identical (:209/:224/:235/:273/:475); [SDMXStructureDataflow.xsd 3.1](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureDataflow.xsd#L63-L70) (`DimensionConstraintType.Dimension` `IDType`; no 3.0 counterpart); `Code.Parent` [3.0 `SingleNCNameIDType`](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXStructureCodelist.xsd#L77) → [3.1 `IDType`](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureCodelist.xsd#L82); `Concept.Parent` [3.0 `SingleNCNameIDType`](https://github.com/sdmx-twg/sdmx-ml/blob/29f1a3d/schemas/SDMXStructureConcept.xsd#L55) → [3.1 `NCNameIDType`](https://github.com/sdmx-twg/sdmx-ml/blob/182248b/schemas/SDMXStructureConcept.xsd#L62) (the identical pattern per `SDMXCommonReferences.xsd`'s own `SingleNCNameIDType` documentation) |
| **Source**   | [Design 0010 — SDMX Core Domain Types](design/0010-sdmx-core-domain-types-design.md) §5.6, §7 |
| **Related**  | [D-0019](#d-0019), [D-0020](#d-0020), [D-0023](#d-0023), [D-0046](#d-0046), [D-0050](#d-0050), [D-0058](#d-0058), [D-0073](#d-0073), [D-0076](#d-0076) |

**Observation**: A local reference identifier is bare `NCNameIDType`/`IDType` content the schema validates independently of whether its target exists, so lexical validity is a local, mechanical property of the lexeme — yet [D-0020](#d-0020)'s reference clause left every such site structural-only, on a rationale ("validating a pointer is redundant if the target was validated") that addresses resolution, not grammar. Eight sites admit lexemes every supported edition rejects: `DimensionRef.id` and `MetadataAttributeUsage.metadata_attribute_ref` (pub fields — even the empty string constructs), the `MeasureRelationship`/`GroupDimensions`/`DimensionConstraint` items (list-level non-empty only), `GroupId` (non-empty only), and `Concept.parent_id`/`Code.parent_id` (unchecked). Meanwhile `ComponentMetadata` NCName-validates the same kind of optional local id, so the model answers one lexical question two ways.

**Decision**: Local reference identifiers validate their site's lexical tier at construction through the [D-0023](#d-0023) validators, stored as plain `String`s — constructor placement, not field newtypes. The construction contract's edition quantifier is universal: a lexeme is a construction error **iff every supported edition's schema rejects it**, so a site whose tier diverges between editions validates at the union of the two grammars — `Code.parent_id` validates `IDType` (3.0 `SingleNCNameIDType` ⊂ 3.1 `IDType`) — and edition-targeted strictness is the writer's obligation, as for every version-specific superset member ([D-0046](#d-0046)). Referential integrity stays deferred exactly as [D-0020](#d-0020)'s Consequences record; maintainable references stay on the URN parse-path contract ([D-0073](#d-0073)). The modelled local-reference sites and their tiers: `NCNameIDType` — `DimensionRef.id`, `MetadataAttributeUsage.metadata_attribute_ref`, `MeasureRelationship` items, `GroupDimensions` items, `Concept.parent_id`; `IDType` — `GroupId`, `DimensionConstraint` items (3.1-only element, so its sole tier), `Code.parent_id` (the union); the constraint selection-node ids are equally within the rule — consequence (6).

**Rationale**: The wire validates every reference lexeme in place, so declaration-side validation covers nothing at the reference site, and a writer emitting an off-tier lexeme produces schema-invalid documents from constructors that cannot fail — the representability class [D-0076](#d-0076) closed for the facets. The union floor is forced, not chosen: a constructor stricter than the loosest edition's grammar could not build values that edition's schema-valid wire carries, breaking round-trip. Constructor placement over field newtypes because the invariant does not travel — no API consumes a statically-valid NCName, uniform newtyping would retype the collection newtypes' delivered `Vec<String>` views, and the guarantee wanted is a property of assembly, which the identifier-validator family ([D-0023](#d-0023)) already provides at every declaration site.

**Consequences**: (1) `DimensionRef`, `MetadataAttributeUsage`, and `Code` cross the §7 carrier→invariant-bearing line (the [D-0023](#d-0023) `Concept`/`Agency` promotion pattern): private fields, fallible `new()`, accessors, Raw-shape `Deserialize`; their carrier classifications in [D-0050](#d-0050)/[D-0058](#d-0058) and D-0023's `Code`-stays-derive-only clause are amended. (2) `GroupId`'s empty special-case dissolves: `Error::EmptyGroupId` is removed and the empty lexeme reports `InvalidIdentifier`, like every identifier site; the collection-emptiness variants (`EmptyMeasureRelationship` and kin) are list-cardinality invariants and stand. (3) The identifier-failure `Display` messages delimit the offending lexeme, so the empty lexeme renders unambiguously. (4) [D-0046](#d-0046)'s Parent-divergence row is re-grounded: carried by union-tier validation, no longer by non-validation. (5) `Item.Parent` and `Organisation.Parent` have no modelled carrier, so nothing validates; a future carrier adopts this rule at its own tier union. (6) The constraint selection-node ids ([D-0051](#d-0051)'s `pub id: String` fields) are local references within this rule: the `CubeRegionKey`/`DataKeyValue` ids validate `SingleNCNameIDType` (the `NCNameIDType` pattern, per the schema's own documentation) and the `ComponentValueSet`/`DataComponentValueSet` ids validate `NestedNCNameIDType` (the dotted tier — a nested metadata-attribute path such as `CONTACT.ADDRESS.STREET` is one lexeme), both editions. (7) The enforcement surface for the writer's per-edition strictness is the writer's design question, deferred until that surface exists; the tier validators stay crate-private.

---
