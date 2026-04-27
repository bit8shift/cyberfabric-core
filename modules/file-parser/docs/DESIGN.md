# Technical Design — File Parser

<!-- toc -->

- [1. Architecture Overview](#1-architecture-overview)
  - [1.1 Architectural Vision](#11-architectural-vision)
  - [1.2 Architecture Drivers](#12-architecture-drivers)
  - [1.3 Architecture Layers](#13-architecture-layers)
- [2. Principles & Constraints](#2-principles--constraints)
  - [2.1 Design Principles](#21-design-principles)
  - [2.2 Constraints](#22-constraints)
- [3. Technical Architecture](#3-technical-architecture)
  - [3.1 Domain Model](#31-domain-model)
  - [3.2 Component Model](#32-component-model)
  - [3.3 API Contracts](#33-api-contracts)
  - [3.4 External Dependencies](#34-external-dependencies)
  - [3.5 Interactions & Sequences](#35-interactions--sequences)
  - [3.6 Database schemas & tables](#36-database-schemas--tables)
- [4. Additional context](#4-additional-context)
  - [Configuration](#configuration)
  - [Error Mapping](#error-mapping)
- [Appendix](#appendix)
  - [Change Log](#change-log)

<!-- /toc -->

## 1. Architecture Overview

### 1.1 Architectural Vision

File Parser is a stateless modkit service module that accepts document uploads (or local file paths) and returns structured content. All format-specific extraction is delegated to a single unified backend (`kreuzberg =4.9.4`, MIT), replacing the previous per-format library approach. The module converts kreuzberg's `ExtractionResult` into a platform-internal IR and optionally renders it as Markdown.

### 1.2 Architecture Drivers

#### Functional Requirements

- Parse PDF, HTML, XLSX/XLS/XLSM/XLSB, PPTX documents into structured blocks
- Preserve headings, paragraphs, lists, tables, code blocks, page breaks, slides, and inline annotations
- Render structured blocks as Markdown
- Enforce path-traversal security for `parse-local` endpoints

#### License Constraint

`kreuzberg` ≥ 4.8.0 uses Elastic License 2.0 (EL-2.0). The dependency is pinned at `=4.9.4`. An explicit exception is declared in `deny.toml` with rationale: CyberFabric's document parsing is not sold as a standalone competing product. Any upgrade requires re-verifying the exception still applies.

### 1.3 Architecture Layers

```
REST API layer  (src/api/)
    ↓
Domain service  (src/domain/service.rs)
    ↓
Parser backend  (src/infra/parsers/kreuzberg_parser.rs)
    ↓
kreuzberg =4.9.4  (ExtractionResult)
    ↓
IR conversion   (src/infra/parsers/ir_convert.rs)
    ↓
Markdown render (src/domain/markdown.rs)
```

## 2. Principles & Constraints

### 2.1 Design Principles

#### Stateless Operation

- [ ] `p1` - **ID**: `cpt-cf-file-parser-principle-stateless`

Parser does not maintain session state. Each request is independent. Temporary files are cleaned up after processing.

#### Format Agnostic API

- [ ] `p1` - **ID**: `cpt-cf-file-parser-principle-format-agnostic`

Unified REST API regardless of input format. Format detection is performed by kreuzberg where possible. Consistent error handling across all formats.

#### Single Backend

- [ ] `p1` - **ID**: `cpt-cf-file-parser-principle-single-backend`

All format-specific logic is delegated to kreuzberg. There are no per-format parser structs; `KreuzbergParser` handles all supported extensions.

### 2.2 Constraints

#### File Size Limit

- [ ] `p2` - **ID**: `cpt-cf-file-parser-constraint-file-size`

Maximum 50 MB per document. Enforced at the API layer via body size limits.

#### Local Path Security

- [ ] `p1` - **ID**: `cpt-cf-file-parser-constraint-local-path-security`

**ID**: [ ] `p1` `fdd-file-parser-constraint-local-path-security-v1`

<!-- fdd-id-content -->
Local file parsing (`parse-local`) validates paths before any filesystem access:
(a) paths containing `..` components are rejected outright;
(b) the requested path is canonicalized (resolving symlinks);
(c) the canonical path must be a descendant of the mandatory `allowed_local_base_dir`.
The module fails to start if `allowed_local_base_dir` is missing or unresolvable.
Violations return HTTP 403 Forbidden. Rejected attempts are logged at `warn` level.
<!-- fdd-id-content -->

#### Supported Formats

- [ ] `p2` - **ID**: `cpt-cf-file-parser-constraint-formats`

**ID**: [ ] `p2` `fdd-file-parser-constraint-formats-v1`

<!-- fdd-id-content -->
PDF, HTML (`html`, `htm`), XLSX/XLS/XLSM/XLSB (spreadsheets), and PPTX (presentations) are supported. Other formats are rejected with HTTP 400.

**Known limitations at kreuzberg 4.9.4**:
- PPTX multi-slide presentations: slides are emitted as `##` headings (not `Slide` nodes); `PageBreak` blocks between slides are not produced. This is an intentional design choice in kreuzberg's PPTX extractor (`extractors/pptx.rs`).
- PPTX tables: the `PptxExtractor` hardcodes `include_structure: false` and rebuilds content from plain text — cells are not pipe-formatted so `parse_markdown_table` does not fire; structured `Table` blocks are not produced.
- DOCX and image formats (PNG, JPG, TIFF) are not supported.
<!-- fdd-id-content -->

#### kreuzberg Version Pin

- [ ] `p1` - **ID**: `cpt-cf-file-parser-constraint-version-pin`

`kreuzberg` is declared as `=4.9.4` in `Cargo.toml`. This prevents `cargo update` from silently upgrading to a newer release with a different or changed license. An explicit `deny.toml` exception permits Elastic-2.0 for this crate at this version. Any upgrade is a deliberate, reviewed action.

## 3. Technical Architecture

### 3.1 Domain Model

Core types (all in `src/domain/`):

| Type | Description |
|---|---|
| `ParsedDocument` | Top-level extraction result: list of `ParsedBlock`, detected `content_type`, optional `source` |
| `ParsedBlock` | Enum: `Heading { level, inlines }`, `Paragraph { inlines }`, `Table(Vec<TableRow>)`, `List { ordered, items }`, `Code { language, content }`, `PageBreak`, `Slide { number, title, blocks }` |
| `Inline` | Enum: `Text { text, style: InlineStyle }`, `Link { target, inlines }` |
| `InlineStyle` | Bitflag-style struct: `bold`, `italic`, `underline`, `strikethrough`, `code` |
| `ParsedSource` | `Upload { filename, content_type }` or `LocalPath { path }` |

### 3.2 Component Model

#### API Layer

- [ ] `p1` - **ID**: `cpt-cf-file-parser-component-rest`

**ID**: [ ] `p1` `fdd-file-parser-component-rest-v1`

<!-- fdd-id-content -->
REST endpoints: `/file-parser/v1/info`, `/file-parser/v1/upload`, `/file-parser/v1/upload/markdown`, `/file-parser/v1/parse-local`, `/file-parser/v1/parse-local/markdown`
<!-- fdd-id-content -->

#### Parser Service

- [ ] `p1` - **ID**: `cpt-cf-file-parser-component-parser-service`

**ID**: [ ] `p1` `fdd-file-parser-component-parser-v1`

<!-- fdd-id-content -->
`ParserService` (`src/domain/service.rs`) — coordinates parsing operations, manages the `KreuzbergParser` backend instance, handles format detection, manages temporary file lifecycle.
<!-- fdd-id-content -->

#### Parser Backend (Kreuzberg)

- [ ] `p1` - **ID**: `cpt-cf-file-parser-component-parser-backend`

**ID**: [ ] `p1` `fdd-file-parser-component-backend-v1`

<!-- fdd-id-content -->
Single `KreuzbergParser` backend (`src/infra/parsers/kreuzberg_parser.rs`) replacing the previous `HtmlParser`, `PdfParser`, `XlsxParser`, and `PptxParser`. Delegates all format-specific extraction to `kreuzberg =4.9.4` with `include_document_structure: true`. Converts kreuzberg's `ExtractionResult` to the platform IR via `result_to_blocks` (`src/infra/parsers/ir_convert.rs`). Inline text annotations (bold, italic, underline, strikethrough, code, hyperlinks) are mapped to `Inline::Text { style }` / `Inline::Link` nodes.
<!-- fdd-id-content -->

#### Markdown Renderer

- [ ] `p1` - **ID**: `cpt-cf-file-parser-component-markdown-renderer`

**ID**: [ ] `p1` `fdd-file-parser-component-markdown-v1`

<!-- fdd-id-content -->
`src/domain/markdown.rs` — converts the `ParsedDocument` IR to Markdown, preserving document structure, tables, and inline formatting.
<!-- fdd-id-content -->

### 3.3 API Contracts

#### REST API

All endpoints accept `Content-Type: application/octet-stream` (binary) or `multipart/form-data`. Upload endpoints return `application/json` (structured blocks) or `text/plain; charset=utf-8` (Markdown).

The `/info` endpoint returns a JSON object with `parsers` key mapping parser name to supported extension list:

```json
{
  "parsers": {
    "kreuzberg": ["pdf", "html", "htm", "xlsx", "xls", "xlsm", "xlsb", "pptx"]
  }
}
```

#### FileParserBackend Trait

Internal Rust trait (`src/domain/parser.rs`) implemented by `KreuzbergParser`:

```rust
pub trait FileParserBackend: Send + Sync {
    fn id(&self) -> &'static str;
    fn supported_extensions(&self) -> &'static [&'static str];
    async fn parse_local_path(&self, path: &Path) -> Result<ParsedDocument, DomainError>;
    async fn parse_bytes(&self, bytes: Bytes, source: ParsedSource) -> Result<ParsedDocument, DomainError>;
}
```

### 3.4 External Dependencies

| Dependency | Version | License | Role |
|---|---|---|---|
| `kreuzberg` | `=4.9.4` | Elastic-2.0 (exception in `deny.toml`) | Unified document extraction backend |
| `pdfium` | bundled via kreuzberg | BSD-3 | PDF rendering (bundled, no separate install) |

### 3.5 Interactions & Sequences

#### Document Upload and Parse

- [ ] `p1` - **ID**: `cpt-cf-file-parser-seq-upload-and-parse`

1. Client uploads document via `POST /file-parser/v1/upload`
2. API layer validates file size (≤ 50 MB) and reads content-type hint from headers
3. `ParserService` locates `KreuzbergParser` (the sole registered backend)
4. `KreuzbergParser::parse_bytes` maps the file extension to a MIME hint and calls `kreuzberg::extract_bytes`
5. kreuzberg extracts content and returns `ExtractionResult` with optional `document_structure`
6. `result_to_blocks` converts the `ExtractionResult` into `Vec<ParsedBlock>`:
   - If `document_structure` is present: walks `DocumentNode` tree, maps nodes to IR blocks, inserts `PageBreak` between consecutive `Slide` root nodes
   - Otherwise: falls back to `result.text` as a single `Paragraph`
7. `ParsedDocument` returned to API layer → serialised as JSON
8. For `/upload/markdown`: `MarkdownRenderer` converts `ParsedDocument` to Markdown string

#### Local File Parse

- [ ] `p1` - **ID**: `cpt-cf-file-parser-seq-local-file-parse`

Same as above but steps 1–3 use `POST /file-parser/v1/parse-local` with a JSON body containing the local path. Path-traversal checks (`..` rejection, canonicalization, base-dir enforcement) are performed in the API layer before the path is passed to `KreuzbergParser::parse_local_path`.

### 3.6 Database schemas & tables

File Parser is stateless and does not own any database tables or persistent storage. No schema migrations are required.

## 4. Additional context

### Configuration

```yaml
# config/server.yaml (relevant keys)
file_parser:
  allowed_local_base_dir: /data/uploads   # required; module fails to start if absent
```

### Error Mapping

| Condition | HTTP status |
|---|---|
| Unsupported file format | 400 Bad Request |
| File too large (> 50 MB) | 413 Payload Too Large |
| Path traversal attempt | 403 Forbidden |
| kreuzberg extraction failure | 500 Internal Server Error |

## Appendix

### Change Log

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2026-02-09 | 0.1.0 | System | Initial DESIGN for cypilot validation |
| 2026-02-17 | 0.2.0 | Security | Removed `/file-parser/v1/parse-url*` endpoints, HTTP client dependency, and `download_timeout_secs` config. Rationale: SSRF risk (issue #525). |
| 2026-02-17 | 0.3.0 | Security | Added path-traversal protections for `parse-local` endpoints: `..` rejection, path canonicalization, `allowed_local_base_dir` enforcement, symlink-escape prevention, `PathTraversalBlocked` error (HTTP 403). Added constraint `fdd-file-parser-constraint-local-path-security-v1`. |
| 2026-04-29 | 0.4.0 | Engineering | Restructured to match cypilot SDLC DESIGN template. Replaced four format-specific parsers with unified `KreuzbergParser` backed by `kreuzberg =4.9.4` (Elastic-2.0). Added `KreuzbergParser` component (`fdd-file-parser-component-backend-v1`). Documented domain model, component model, API contracts, and interaction sequences. |
