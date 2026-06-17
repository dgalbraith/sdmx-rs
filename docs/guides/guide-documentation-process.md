# Documentation Process Guide

Last Updated: 2026-05-24

## Overview

This guide explains what guides are, who should write them, when they're used, and how the guide documentation system works in sdmx-rs.

Guides are **step-by-step tutorials and walkthroughs** designed for learning. They complement ADRs (architectural decisions), design documents (exploratory design), and rustdoc (API reference).

---

## Prerequisites

None. This guide is an introduction to the guide documentation system itself.

---

## What Are Guides?

### Definition

A guide is a **mutable, user-facing tutorial** that teaches readers how to accomplish a specific task with sdmx-rs.

**Guides are for**:
- Learning through examples ("How do I query structures?")
- Step-by-step walkthroughs ("Building a client from scratch")
- Troubleshooting common problems ("Why does my builder not compile?")
- Explaining concepts in context ("What is typestate validation?")

**Guides are not for**:
- Recording architectural decisions (use [ADRs](../adr/README.md) instead)
- Exploring design trade-offs (use [Design Documents](../design/README.md) instead)
- Documenting API signatures (use [rustdoc](https://docs.rs/sdmx-rs) instead)

### Key Characteristics

| Aspect         | ADRs                          | Design Docs                 | Guides                  | Rustdoc            |
|----------------|-------------------------------|-----------------------------|-------------------------|--------------------|
| **Purpose**    | Record decisions              | Explore design              | Teach tasks             | Document API       |
| **Audience**   | Developers                    | Architects                  | Users                   | All                |
| **Mutable?**   | ❌ No (immutable)             | ✅ Yes (mutable)            | ✅ Yes (mutable)        | Depends            |
| **Versioned?** | ✅ Yes (numbered)             | ✅ Yes (numbered)           | ✅ Yes (slugs)          | ✅ Yes (per crate) |
| **Structure**  | Context/Decision/Consequences | Problem/Design/Alternatives | Overview/Steps/Examples | Comments + tests   |

---

## When to Write a Guide

Write a guide when you need to explain **how to do something** with sdmx-rs.

Examples of guide topics:
- Querying data structures (codelists, dataflows, DSDs)
- Streaming observation data with error recovery
- Using typestate builders for compile-time validation
- Advanced query patterns (filtering, pivoting, windowing)
- Integrating sdmx-rs into production systems
- Performance tuning and benchmarking

---

## Guide Structure

Each guide follows this template:

1. **Overview** — What is this guide about and who should read it?
2. **Prerequisites** — What knowledge or setup is required?
3. **Step-by-Step** — Clear, numbered steps with explanations and examples
4. **Examples** — Self-contained, runnable code samples
5. **Troubleshooting** — Common problems and solutions
6. **Next Steps** — Related guides, design docs, or API references
7. **Notes** (optional) — Version compatibility, caveats, future improvements

See [Template](./templates/template.md) for the full structure.

---

## Creating and Managing Guides

### Creating a New Guide

Use the `just guide` command:

```bash
just guide "Working with Typestate Builders"
```

This will:
1. Sanitise the title and create a slug (e.g., `working-with-typestate-builders`)
2. Generate the filename: `docs/guides/working-with-typestate-builders.md`
3. Render the template with today's date
4. Register the file in `.gitignore`

The guide is now ready to edit.

### Editing a Guide

Open the file and fill in each section:

```bash
$EDITOR docs/guides/working-with-typestate-builders.md
```

Guides can be updated at any time—they're mutable documents.

### Renaming a Guide

If you want to improve the title:

```bash
just guide-rename old-slug "New Title"
```

This updates the filename and `.gitignore` ledger automatically.

### Removing a Guide

If a guide is no longer relevant:

```bash
just guide-remove old-slug
```

This removes the file and cleans up `.gitignore`.

---

## Guide Lifecycle

Unlike ADRs (Proposed → Accepted → Superseded), guides don't have formal states. Instead:

- **Draft** — Unpublished or incomplete (leave as-is; readers can see it)
- **Published** — Complete and ready for users (no special marker; complete docs are published)
- **Updated** — Improved or expanded (update the "Last Updated" date)
- **Archived** — No longer relevant (remove the file or add a deprecation note)

---

## Referencing Guides

From other documents, link to guides like this:

```markdown
See [Guide Documentation Process](../guides/guide-documentation-process.md) for an explanation on the purpose of a guide.
```

From rustdoc (in code comments):

```rust
/// See the [Guide Documentation Process](https://docs.rs/sdmx-rs/latest/sdmx_rs/guides/guide-documentation-process.md)
/// for an explanation on the purpose of a guide.
pub fn example() { }
```

---

## Quality Standards

Guides should:

✅ **Be complete** — Reader can follow all steps without getting stuck
✅ **Include runnable examples** — Point to or embed working code
✅ **Be tested** — Examples compile and run as documented
✅ **Be discoverable** — Linked from docs/README.md and relevant API docs
✅ **Be maintainable** — Keep step count under 10; break complex tasks into multiple guides

---

## Next Steps

- **Writing your first guide?** Start with the [Guide Template](./templates/template.md)
- **Want examples?** See guides in this directory
- **New to sdmx-rs?** Start with the [User Guide](../user/README.md)
- **Contributing code?** See [CONTRIBUTING.md](../../CONTRIBUTING.md)

---

## See Also

- [ADR-0001: Architecture Decision Records](../adr/0001-record-architecture-decisions.md): Architectural decision process
- [Design Document Process](../design/0001-design-documentation-process.md): Design exploration and RFC process
- [practices.md](../dev/practices.md): Code style, naming, documentation standards
- [User Guide](../user/README.md): Getting started with sdmx-rs
