# PRD — File Parser

<!-- toc -->

- [1. Overview](#1-overview)
  - [1.1 Purpose](#11-purpose)
  - [1.2 Background / Problem Statement](#12-background--problem-statement)
  - [1.3 Goals (Business Outcomes)](#13-goals-business-outcomes)
  - [1.4 Glossary](#14-glossary)
- [2. Actors](#2-actors)
  - [2.1 Human Actors](#21-human-actors)
  - [2.2 System Actors](#22-system-actors)
- [3. Operational Concept & Environment](#3-operational-concept--environment)
- [4. Scope](#4-scope)
  - [4.1 In Scope](#41-in-scope)
  - [4.2 Out of Scope](#42-out-of-scope)
- [5. Functional Requirements](#5-functional-requirements)
  - [Document Upload](#document-upload)
  - [Format Support](#format-support)
  - [Content Extraction](#content-extraction)
  - [Markdown Rendering](#markdown-rendering)
  - [Local Path Security](#local-path-security)
- [6. Non-Functional Requirements](#6-non-functional-requirements)
  - [Performance](#performance)
  - [Scalability](#scalability)
  - [Reliability](#reliability)
  - [6.1 NFR Exclusions](#61-nfr-exclusions)
- [7. Public Library Interfaces](#7-public-library-interfaces)
  - [7.1 Public API Surface](#71-public-api-surface)
  - [7.2 External Integration Contracts](#72-external-integration-contracts)
- [8. Use Cases](#8-use-cases)
  - [UC-001: Upload and Parse Document](#uc-001-upload-and-parse-document)
  - [UC-002: Parse Local File](#uc-002-parse-local-file)
- [9. Acceptance Criteria](#9-acceptance-criteria)
- [10. Dependencies](#10-dependencies)
- [11. Assumptions](#11-assumptions)
- [12. Risks](#12-risks)
- [Appendix](#appendix)
  - [Change Log](#change-log)

<!-- /toc -->

## 1. Overview

### 1.1 Purpose

File Parser provides document parsing and content extraction capabilities for the HyperSpot platform. It accepts documents in common office and web formats and returns structured content (text, headings, lists, tables, inline annotations) suitable for AI workflows or human-readable Markdown output.

### 1.2 Background / Problem Statement

Platform modules — most notably the Chat Engine and LLM Gateway — need to process user-uploaded documents as grounding material for AI responses. These documents arrive as binary uploads in varying formats (PDF, HTML, spreadsheets, presentations).

The module previously used four separate parsing libraries (`tl` for HTML, `pdf-extract` for PDF, `calamine` for Excel, `pptx-to-md` for PowerPoint). This resulted in fragmented format-specific logic, inconsistent IR quality, a temp-file workaround in the PDF path, and four separate dependency trees with differing licenses.

Replacing all four libraries with `kreuzberg =4.9.4` (Elastic-2.0 licensed) unifies the extraction pipeline, removes the temp-file hack, and provides a richer document structure via a single API.

### 1.3 Goals (Business Outcomes)

- Provide a single, unified REST API for document content extraction regardless of input format
- Return structured content that preserves document semantics (headings, lists, tables, inline annotations)
- Produce Markdown output suitable for injection into LLM prompts
- Enable downstream modules to process documents without understanding format-specific details
- Parse common formats with ≥ 95% accuracy; respond in < 5 s for documents < 10 MB

### 1.4 Glossary

| Term | Definition |
|------|------------|
| Parsed block | A unit of structured document content: `Heading`, `Paragraph`, `Table`, `List`, `Code`, `PageBreak`, or `Slide` |
| Inline annotation | Span-level styling within text: bold, italic, underline, strikethrough, code, or hyperlink |
| IR (Intermediate Representation) | Internal data model produced by the parser backend and consumed by the Markdown renderer |
| kreuzberg | Third-party Rust crate (`=4.9.4`, MIT) used as the unified parser backend |
| `parse-local` | Endpoint family that parses files already present on the server filesystem |
| `allowed_local_base_dir` | Mandatory config field constraining which filesystem paths `parse-local` may access |

## 2. Actors

### 2.1 Human Actors

#### API User

**ID**: `fdd-file-parser-actor-api-user`

<!-- fdd-id-content -->
**Role**: End user or developer who uploads documents and receives parsed content or Markdown.
**Needs**: Upload a document via REST, receive structured text content or Markdown in the response.
<!-- fdd-id-content -->

### 2.2 System Actors

#### Consumer Module

**ID**: `fdd-file-parser-actor-consumer`

<!-- fdd-id-content -->
**Role**: Internal platform module (e.g., Chat Engine) that calls File Parser programmatically as part of a document-processing workflow.
**Needs**: Reliable structured content extraction from files on the server filesystem (`parse-local`) or from binary payloads.
<!-- fdd-id-content -->

## 3. Operational Concept & Environment

> **Note**: Project-wide runtime, OS, architecture, and lifecycle policy are defined in the root PRD. Only module-specific deviations are documented here.

File Parser runs as a stateless HTTP service within the HyperSpot platform. Each request is fully self-contained: the module accepts an uploaded file (or a local path), extracts content, and returns the result without persisting any state. Temporary files are cleaned up after each request.

The module requires an `allowed_local_base_dir` config entry to be set at startup. If the value is missing or unresolvable, the module fails to start.

## 4. Scope

### 4.1 In Scope

- Binary and multipart file upload endpoints returning JSON-structured content
- Markdown rendering endpoint returning extracted content as Markdown
- Local-path parsing endpoints for server-side files
- Format support: PDF, HTML (`html`, `htm`), spreadsheets (XLSX / XLS / XLSM / XLSB), presentations (PPTX)
- Preservation of document structure: headings, paragraphs, lists, tables, code blocks, page breaks, slides
- Inline annotation extraction: bold, italic, underline, strikethrough, inline code, hyperlinks
- Parser info endpoint listing supported extensions per parser

### 4.2 Out of Scope

- OCR for scanned or image-only documents (future enhancement — kreuzberg `ocr` feature + Tesseract; separate PR)
- DOCX parsing (not supported by `kreuzberg =4.9.4` without additional feature flags)
- Image format parsing (PNG, JPG, TIFF)
- Document editing or modification
- Long-term document storage
- Format conversion other than Markdown
- URL-based document fetching (removed due to SSRF risk, issue #525)

## 5. Functional Requirements

### Document Upload

- [ ] `p1` - **ID**: `cpt-cf-file-parser-fr-upload`

**ID**: [ ] `p1` `fdd-file-parser-fr-upload-v1`

<!-- fdd-id-content -->
System SHALL support binary file upload and multipart form upload with optional Markdown rendering. System SHALL handle documents up to 50 MB. Requests exceeding the size limit SHALL be rejected with HTTP 413.

**Actors**: `fdd-file-parser-actor-api-user`
<!-- fdd-id-content -->

### Format Support

- [ ] `p1` - **ID**: `cpt-cf-file-parser-fr-formats`

**ID**: [ ] `p1` `fdd-file-parser-fr-formats-v1`

<!-- fdd-id-content -->
System SHALL support parsing PDF, HTML (`html`, `htm`), and Office formats: XLSX/XLS/XLSM/XLSB (spreadsheets) and PPTX (presentations) via `kreuzberg =4.9.4`.

**Known limitations at kreuzberg 4.9.4**:
- PPTX multi-slide presentations: slides are emitted as headings rather than `Slide` nodes; `PageBreak` blocks between slides are not produced.
- PPTX tables: structured `Table` blocks are not produced; table cell content is extracted as paragraphs.

**Actors**: `fdd-file-parser-actor-api-user`
<!-- fdd-id-content -->

### Content Extraction

- [ ] `p1` - **ID**: `cpt-cf-file-parser-fr-extraction`

**ID**: [ ] `p1` `fdd-file-parser-fr-extraction-v1`

<!-- fdd-id-content -->
System SHALL extract text content and preserve document structure (headings, paragraphs, lists, tables, code blocks, page breaks, slides). Inline text annotations (bold, italic, underline, strikethrough, code, hyperlinks) SHALL be preserved in the parsed output.

**Actors**: `fdd-file-parser-actor-api-user`
<!-- fdd-id-content -->

### Markdown Rendering

- [ ] `p1` - **ID**: `cpt-cf-file-parser-fr-markdown`

**ID**: [ ] `p1` `fdd-file-parser-fr-markdown-v1`

<!-- fdd-id-content -->
System SHALL convert extracted document structure to Markdown format, preserving headings, lists, formatting, tables, and code blocks.

**Actors**: `fdd-file-parser-actor-api-user`
<!-- fdd-id-content -->

### Local Path Security

- [ ] `p1` - **ID**: `cpt-cf-file-parser-fr-local-path-security`

**ID**: [ ] `p1` `fdd-file-parser-fr-local-path-security-v1`

<!-- fdd-id-content -->
System SHALL reject local file paths containing `..` traversal components. System SHALL require a mandatory `allowed_local_base_dir` configuration; the module SHALL fail to start if this field is missing or the path cannot be resolved. System SHALL canonicalize the requested path (resolving symlinks) and reject paths that do not fall under the base directory. Rejected requests SHALL return HTTP 403 and be logged at `warn` level.

**Actors**: `fdd-file-parser-actor-api-user`, `fdd-file-parser-actor-consumer`
<!-- fdd-id-content -->

## 6. Non-Functional Requirements

### Performance

- [ ] `p1` - **ID**: `cpt-cf-file-parser-nfr-response-time`

**ID**: [ ] `p1` `fdd-file-parser-nfr-response-time-v1`

<!-- fdd-id-content -->
System SHALL respond in < 5 s for documents < 10 MB and < 30 s for documents < 50 MB.
<!-- fdd-id-content -->

### Scalability

- [ ] `p1` - **ID**: `cpt-cf-file-parser-nfr-concurrency`

**ID**: [ ] `p1` `fdd-file-parser-nfr-concurrency-v1`

<!-- fdd-id-content -->
System SHALL support 100 concurrent parsing requests.
<!-- fdd-id-content -->

### Reliability

- [ ] `p1` - **ID**: `cpt-cf-file-parser-nfr-availability`

**ID**: [ ] `p1` `fdd-file-parser-nfr-availability-v1`

<!-- fdd-id-content -->
System SHALL maintain 99.9% uptime SLA.
<!-- fdd-id-content -->

### 6.1 NFR Exclusions

| NFR | Reason excluded |
|-----|-----------------|
| OCR accuracy SLA | OCR is out of scope for this version |
| DOCX / image parsing SLA | Those formats are out of scope for this version |

## 7. Public Library Interfaces

### 7.1 Public API Surface

REST endpoints exposed by the module:

| Endpoint | Method | Description |
|---|---|---|
| `/file-parser/v1/info` | GET | List available parsers and supported extensions |
| `/file-parser/v1/upload` | POST | Upload and parse a document; returns JSON |
| `/file-parser/v1/upload/markdown` | POST | Upload and parse a document; returns Markdown |
| `/file-parser/v1/parse-local` | POST | Parse a server-local file; returns JSON |
| `/file-parser/v1/parse-local/markdown` | POST | Parse a server-local file; returns Markdown |

### 7.2 External Integration Contracts

No external service contracts. The module depends only on the `kreuzberg` Rust library (in-process) and the host filesystem for `parse-local` endpoints.

## 8. Use Cases

### UC-001: Upload and Parse Document

**ID**: [ ] `p1` `fdd-file-parser-usecase-upload-parse-v1`

<!-- fdd-id-content -->
User uploads a document (PDF, HTML, XLSX, PPTX) and receives parsed content with structured blocks and optional Markdown rendering.

**Actors**: `fdd-file-parser-actor-api-user`
**Preconditions**: Document is in a supported format and ≤ 50 MB.
**Postconditions**: Structured blocks (headings, paragraphs, tables, etc.) returned in JSON or Markdown.
<!-- fdd-id-content -->

### UC-002: Parse Local File

**ID**: [ ] `p1` `fdd-file-parser-usecase-local-parse-v1`

<!-- fdd-id-content -->
Consumer module requests parsing of a file already present on the server filesystem.

**Actors**: `fdd-file-parser-actor-consumer`
**Preconditions**: File exists under `allowed_local_base_dir`.
**Postconditions**: Structured content returned; path traversal attempts rejected with HTTP 403.
<!-- fdd-id-content -->

## 9. Acceptance Criteria

| Criterion | Condition |
|---|---|
| Supported formats parsed | PDF, HTML, XLSX, PPTX upload returns non-empty structured blocks |
| Markdown rendering | Upload-markdown endpoint returns valid Markdown preserving headings and tables |
| Path traversal rejection | Requests with `..` components return HTTP 403 |
| Unknown format rejection | Upload of unsupported extension returns HTTP 400 |
| Parser info endpoint | `/info` returns `kreuzberg` parser with correct extension list |
| File size limit | Upload > 50 MB returns HTTP 413 |

## 10. Dependencies

| Dependency | Type | Notes |
|---|---|---|
| `kreuzberg =4.9.4` | External library (Elastic-2.0, exception in `deny.toml`) | Pinned with `=` to prevent silent upgrades; license exception documented in `deny.toml` |
| `pdfium` | Bundled native library | Included via `bundled-pdfium` feature; no separate install required |
| Host filesystem | Runtime | Required for `parse-local` endpoints |
| modkit HTTP framework | Internal | REST endpoint registration and body-size enforcement |

## 11. Assumptions

- `kreuzberg =4.9.4` will remain available on crates.io for the foreseeable future.
- The `bundled-pdfium` feature provides a sufficiently up-to-date PDFium for production PDF parsing.
- 50 MB is a sufficient document size limit for current use cases; revision requires an NFR update.
- Consumer modules calling `parse-local` are trusted to supply valid paths within `allowed_local_base_dir`.

## 12. Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| kreuzberg 4.8.0 relicensed to Elastic-2.0 | Done (already happened) | High — incompatible with platform license policy | Version pinned with `=`; upgrade requires explicit license review |
| PPTX multi-slide / table limitations persist in future kreuzberg releases | Medium | Low — documented as known limitation | Tests marked `#[ignore]` with strict assertions; re-evaluate on upgrade |
| PDFium bundled binary lags security patches | Low | Medium | Track pdfium releases; update via kreuzberg upgrade when license permits |
| Large document OOM under concurrency | Low | Medium | 50 MB limit enforced at API layer; monitor memory usage in production |

## Appendix

### Change Log

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2026-02-09 | 0.1.0 | System | Initial PRD for cypilot validation |
| 2026-02-17 | 0.2.0 | Security | Removed URL parsing capability (use case `fdd-file-parser-usecase-url-parse-v1`, FR `fdd-file-parser-fr-url-v1`). Rationale: SSRF vulnerability (issue #525). |
| 2026-02-17 | 0.3.0 | Security | Added FR `fdd-file-parser-fr-local-path-security-v1` — path-traversal protections for `parse-local`. Rationale: prevent arbitrary file read via path traversal (issue #525). |
| 2026-04-29 | 0.4.0 | Engineering | Restructured to match cypilot SDLC PRD template. Replaced four format-specific parsing libraries (`tl`, `pdf-extract`, `calamine`, `pptx-to-md`) with `kreuzberg =4.9.4` (Elastic-2.0-licensed). Removed DOCX and image format support. Added HTML parsing. Added inline annotation extraction. Documented kreuzberg 4.9.4 PPTX limitations. |
