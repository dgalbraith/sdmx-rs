# 18. Content-Type Negotiation and Parser Routing

Date: 2026-05-21

## Status

Accepted, extended by ADR-0019

---

## Context

The `sdmx-client` crate must communicate format preferences to SDMX REST endpoints via `Accept` headers and then route the server's response to the correct parser in `sdmx-parsers` based on the response `Content-Type` header.

SDMX REST endpoints can serve multiple wire formats for the same logical request:

* **SDMX-ML** (`application/vnd.sdmx.structure+xml` / `application/vnd.sdmx.data+xml`)
* **SDMX-JSON** (`application/vnd.sdmx.structure+json` / `application/vnd.sdmx.data+json`)
* **SDMX-CSV** (`application/vnd.sdmx.data+csv`) — data queries only

The choice of format has material performance implications:

* **SDMX-CSV** is the most compact wire format for observation data. It has minimal framing overhead, is row-oriented, and maps directly to a streaming row iterator. For large datasets it is typically the fastest to parse and the smallest to transfer.
* **SDMX-JSON** is compact and widely supported. Streaming is possible via `serde_json`'s iterator API but JSON's nested structure has higher framing overhead than CSV for flat observation data.
* **SDMX-ML** is the most verbose format. It is the most broadly supported across older and less capable endpoints and carries the richest structural metadata, but imposes the highest parse overhead.

The format priority ordering, the exact `Accept` header values, and the response routing logic are architectural decisions that must be recorded explicitly. Without a record, this logic risks being re-decided ad-hoc during implementation.

## Decision Drivers

* **Performance**: Minimise parse overhead and wire transfer size for the common data retrieval use case.
* **Broad Compatibility**: Maintain fallback to SDMX-ML for endpoints that do not support newer formats, which are common among older national statistical office deployments.
* **Deterministic Routing**: The parser selected must always match the format returned; ambiguous or incorrect routing is a correctness failure, not just a performance issue.
* **Testability**: The Accept header construction and response routing logic must be independently testable without a live endpoint.
* **SDMX Spec Alignment**: `Accept` header values must use the official IANA-registered SDMX MIME types.

---

## Options Considered

### Option A — Hardcoded single format per query type

Issue requests with a single, fixed `Accept` value (e.g., always `application/vnd.sdmx.data+csv` for data queries).

**Pros**:

* Simple routing: response format is always known in advance.

**Cons**:

* Brittle against endpoints that do not support the requested format and return `406 Not Acceptable` instead of falling back.
* No mechanism to override per-query if a caller needs a specific format.

**Verdict**: Rejected.

### Option B — Priority-ordered `Accept` header with `q` weights

Construct a priority-ordered `Accept` header using RFC 7231 quality values (`q=`). The client expresses a ranked preference; the endpoint selects the highest-priority format it supports. The response `Content-Type` header is then used to route to the correct parser.

**Pros**:

* Maximises performance on capable endpoints (CSV or JSON) while gracefully degrading to XML on older ones.
* Standard HTTP mechanism — no SDMX-specific negotiation logic required.
* Response routing is unambiguous: parse the `Content-Type` of the actual response, not the requested format.

**Cons**:

* Slightly more complex `Accept` header construction.
* The client must handle the case where an endpoint ignores `Accept` and returns an unexpected format.

**Verdict**: Accepted.

### Option C — User-configurable format preference

Expose a `Format` enum on the client builder allowing callers to override the default priority ordering.

**Pros**:

* Gives advanced users full control over wire format.

**Cons**:

* Premature API surface before the format landscape is fully implemented. Can be layered on top of Option B once all parsers exist.

**Verdict**: Deferred to post-Phase-2 once all three parsers are available.

---

## Decision

**The `sdmx-client` will issue priority-ordered `Accept` headers using RFC 7231 `q=` weights. The format priority differs by query type. The actual response `Content-Type` header is used to select the parser — not the requested format.**

### Format Priority Ordering

#### Data Queries

```
Accept: application/vnd.sdmx.data+csv;version=2.1.0;q=1.0,
        application/vnd.sdmx.data+json;version=2.1.0;q=0.8,
        application/vnd.sdmx.data+xml;version=3.1.0;q=0.5
```

CSV is preferred first for data queries. It is the most compact and most efficiently streamable format for flat observation data. JSON is a capable second choice. XML is the compatibility fallback.

> **Note on XML version**: SDMX-ML data uses the SDMX specification version (3.x), not the JSON format version (2.x). Version 3.1.0 aligns with SDMX 3.1; 3.0.0 is acceptable for SDMX 3.0 endpoints.

**Why CSV over JSON for data?**

SDMX-CSV encodes one observation per row with dimension values as flat columns. There is no JSON object nesting overhead, no repeated key strings per observation, and no bracket/comma framing. For a 1 M-row dataset, the wire-size difference is substantial. The row-oriented structure also maps directly to a streaming iterator without a push-parser state machine. The only trade-off is that CSV carries less structural metadata than JSON or XML inline — but for data retrieval, the DSD is fetched separately and the observation values are what matters.

**Why not JSON over CSV?**

SDMX-JSON is an excellent format and the right default for metadata queries where structural richness matters. For raw observation data, the verbosity of JSON object keys per observation (repeated for every row) means CSV will consistently outperform it on both transfer size and parse speed. This is the same reasoning used by columnar database formats (Parquet, Arrow) vs. row-oriented JSON.

#### Metadata / Structure Queries

```
Accept: application/vnd.sdmx.structure+json;version=2.1.0;q=1.0,
        application/vnd.sdmx.structure+xml;version=3.1.0;q=0.8
```

JSON is preferred for structure queries. SDMX-CSV is not defined for structure responses. JSON is more compact than XML for structural metadata; XML is the fallback for broad compatibility.

> **Note on versions**: Structure queries use JSON v2.1.0 (the latest SDMX-JSON format version, corresponding to SDMX 3.0+) and XML v3.1.0 (the latest SDMX specification version). For endpoints supporting only SDMX 3.0, XML v3.0.0 is acceptable; for legacy SDMX 2.1 endpoints, use `version=2.1` for XML. JSON v2.0.0 is also valid for SDMX 3.0 endpoints.

### Response Routing

On receipt of a response, the `Content-Type` header determines the parser:

| Response `Content-Type` (prefix match) | Parser                                |
|----------------------------------------|---------------------------------------|
| `application/vnd.sdmx.data+csv`        | `sdmx-parsers` CSV path               |
| `application/vnd.sdmx.data+json`       | `sdmx-parsers` JSON path              |
| `application/vnd.sdmx.data+xml`        | `sdmx-parsers` XML path               |
| `application/vnd.sdmx.structure+json`  | `sdmx-parsers` JSON path              |
| `application/vnd.sdmx.structure+xml`   | `sdmx-parsers` XML path               |
| Anything else                          | `ClientError::UnsupportedContentType` |

If the server returns a `Content-Type` not in this table, the client returns a typed error rather than attempting to parse the body. This is a hard correctness boundary.

### SDMX Version Conveyance for CSV

Unlike SDMX-ML and SDMX-JSON, SDMX-CSV payloads carry no version envelope. The SDMX version is conveyed to the CSV parser constructor as an explicit parameter derived from the `version=` parameter in the response `Content-Type` header (if present) or from the version negotiated via the `Accept` header.

---

## Consequences

* **Positive**: Data queries default to SDMX-CSV, delivering maximum throughput on capable endpoints with graceful degradation to JSON then XML on older ones.
* **Positive**: Response routing is deterministic and testable: given a `Content-Type` string, the correct parser is always selected without ambiguity.
* **Positive**: The priority ordering is recorded here as the single source of truth; `sdmx-client` implementation derives the `Accept` header mechanically from this table.
* **Negative**: The client must handle `406 Not Acceptable` responses and potentially retry with a lower-priority `Accept` value, or propagate the error. Retry strategy is deferred to Phase 3.
* **Negative**: If a server ignores the `Accept` header and returns an unexpected `Content-Type`, the client returns an error rather than silently attempting a best-effort parse. This is intentional but may surprise callers expecting lenient handling.
* **Neutral**: User-configurable format override (Option C) is deferred until all three parsers exist post-Phase 2.

---

## References

### Authoritative SDMX REST Specification
* [SDMX REST API Schema Documentation](https://github.com/sdmx-twg/sdmx-rest/blob/master/doc/schema.md) — Canonical media types for structure and data queries, including all supported versions
  * Structure queries: `application/vnd.sdmx.structure+json;version=2.1.0|2.0.0` and `application/vnd.sdmx.structure+xml;version=3.1.0|3.0.0`
  * Data queries: `application/vnd.sdmx.data+json;version=2.1.0|2.0.0`, `application/vnd.sdmx.data+xml;version=3.1.0|3.0.0`, `application/vnd.sdmx.data+csv;version=2.1.0|2.0.0`

### Format Specifications
* [SDMX-JSON Schemas](https://json.sdmx.org/) — Official SDMX-JSON format versions (1.0, 2.0.0, 2.1.0) and structure/data/metadata schemas
* [SDMX Standards Overview](https://sdmx.org/standards-2/) — SDMX 3.0 and 3.1 specification versions and format portfolio
* [SDMX-CSV Field Guide](https://github.com/sdmx-twg/sdmx-csv/blob/master/data-message/docs/sdmx-csv-field-guide.md) — SDMX-CSV 2.0/2.1 format specification for data payloads

### Implementation Guidance
* [ECB Data Portal Content Negotiation](https://data.ecb.europa.eu/help/api/content-negotiation) — Practical media type examples and version defaults
* [World Bank SDMX API Guide](https://datahelpdesk.worldbank.org/knowledgebase/articles/1886701-sdmx-api-queries) — Real-world SDMX REST endpoint implementation

### Related ADRs & Code
* ADR-0008 — `ConstraintModel` version routing; version conveyance for CSV is out-of-band (this ADR)
* ADR-0009 — `quick-xml` and `serde_json` streaming parsers (XML/JSON paths referenced in routing table)
* ADR-0017 — SDMX-CSV stream parsing strategy
* [Design Document 0008](../design/0008-target-version-policy-for-serialisation.md) — Target version policy for serialisation (the `TargetVersion` parameter mirrors the version conveyance problem on the outbound path)
* `crates/sdmx-client/src/lib.rs` — Content-Type negotiation implementation
* RFC 7231 §5.3.2 — `Accept` header quality weight specification
