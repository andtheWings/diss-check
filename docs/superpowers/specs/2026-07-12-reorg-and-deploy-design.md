# ScholarPress Reorganization & GHCR Deployment

Date: 2026-07-12

## 1. Objective

Reorganize the fragmented repository structure into a clean, language-aligned architecture with pre-built Docker images distributed via GitHub Container Registry (GHCR). Move from mixed-purpose repositories to strict separation of concerns.

## 2. Repository Architecture (Post-Reorg)

| Repo | Role | Content |
|------|------|---------|
| `scholarpress-catalog` | Pure data registry | YAML specs, Typst templates, test fixtures, download scripts. **Unchanged.** |
| `scholarpress-publish-ui` | Next.js 15 web app | Extracted from `publish/web/`. Zero Rust code. Dark themed chat UI with AI SDK v7 tool orchestration. |
| `scholarpress-backend` | Rust monorepo (Cargo workspace) | All Rust: `sp-extract` lib, `sp-validate` lib, `sp-typst` lib, `publish-service` app, `scholarpress-cli` app. Merged from check, publish/rust-doc-service, and cli. |
| `scholarpress-deliver` | Orchestration repo | User-facing `docker-compose.yml` that pulls GHCR images. No source code. |

**Git history**: `git filter-repo` extracts directories with full commit history preserved:
- `publish/web/` → `scholarpress-publish-ui`
- `check/src/` → `scholarpress-backend/crates/sp-validate/`
- `cli/src/` → `scholarpress-backend/apps/scholarpress-cli/`
- `publish/rust-doc-service/` → `scholarpress-backend/apps/publish-service/` + `crates/sp-extract/` + `crates/sp-typst/`

## 3. Cargo Workspace Structure

```
scholarpress-backend/
  Cargo.toml              # [workspace] members = ["crates/*", "apps/*"]
  Cargo.lock

  crates/
    sp-extract/           # Unified PDF + DOCX extraction
      Cargo.toml          # deps: pdf_oxide, zip, quick-xml, roxmltree, regex
      src/
        lib.rs            # extract_pdf(bytes) -> ParsedDocument, extract_pdf_spans(bytes) -> Document
        pdf.rs            # pdf_oxide integration (merged from both codebases)
        docx.rs           # zip + roxmltree DOCX parser
        document.rs       # ParsedDocument, ParsedPage, ParsedParagraph, TextSpan, Heading, Metadata
        chunker.rs        # Sliding-window text chunker
        heading.rs        # Heuristic heading detection with scoring

    sp-validate/          # PDF formatting validation (current scholarpress-check)
      Cargo.toml          # deps: sp-extract, pdf_oxide, serde, serde_yaml, regex
      src/
        lib.rs            # Public modules
        engine.rs         # CheckOptions, run_checks(spec, pdf_bytes) -> Vec<CheckResult>
        spec.rs           # InstitutionSpec, CheckDef, load_spec(path)
        report.rs         # build_report(), format_text(), format_json()
        calibration.rs    # run_calibration(), CalibrationReport
        checkers/
          mod.rs          # Checker trait, Status, EvidenceItem, CheckResult, REGISTRY
          layout.rs       # margins, margin_symmetry (percentile-based)
          typography.rs   # font_size, font_weight, font_family, justification
          structure.rs    # section_presence, section_order, page_numbers, headings, etc.
          content.rs      # boilerplate_match (70% threshold), committee_order, toc_title_parity
          title_page.rs   # all_caps, clause_centered, clause_spacing
          optional_pages.rs
          footnotes.rs
          sections.rs
          toc_details.rs

    sp-typst/             # Typst compilation (native library, not subprocess)
      Cargo.toml          # deps: typst, typst-pdf
      src/
        lib.rs            # compile(source: &str, root: Option<&Path>) -> Result<Vec<u8>> (PDF bytes)
        template.rs       # load_template(), render_template() with variable substitution

  apps/
    publish-service/      # Axum web server (port 4000)
      Cargo.toml          # deps: sp-extract, sp-validate, sp-typst, axum, tokio, tower-http, tracing, etc.
      Dockerfile          # Multi-stage build from workspace root
      src/
        main.rs           # Server entry point
        lib.rs            # App bootstrap, middleware (request-id, CORS, tracing), router
        config.rs         # AppConfig { port, catalog_path: PathBuf }
        error.rs          # AppError enum + IntoResponse
        institutions/
          mod.rs          # Registry (loads from CATALOG_PATH)
        routes/
          mod.rs          # Router definition (7 endpoints)
          extract.rs      # POST /extract
          compile.rs      # POST /compile
          validate.rs     # POST /validate
          institutions.rs # GET /institutions
          spec.rs         # GET /institutions/:id/spec
          template.rs     # GET /institutions/:id/template
      tests/              # Integration tests

    scholarpress-cli/     # CLI binary
      Cargo.toml          # deps: sp-extract, sp-validate, clap, serde_json
      src/
        main.rs           # Subcommands: check, calibrate, dump-extract, dump-spans
```

### Key design decisions

- **`sp-extract`** unifies both PDF extraction codebases. Exposes two modes: `extract_pdf()` returns `ParsedDocument` (paragraph-level, used by publish-service) and `extract_pdf_spans()` returns `Document` (word-level, used by sp-validate checkers). One pdf_oxide dependency, one `build_word()`, one coordinate-flip implementation.
- **`sp-validate`** is the renamed `scholarpress-check` library. Gains a `pdf_bytes` codepath via sp-extract. All 36 checkers, 83 tests, and calibration workflow preserved. Institution specs read from catalog via `CATALOG_PATH`.
- **`sp-typst`** replaces the `typst compile` subprocess shell-out with the `typst` crate as a native dependency. Exposes `compile(source: &str, root: Option<&Path>) -> Result<Vec<u8>>`. Template loading and variable substitution live here.
- **`publish-service`** imports all three crates locally. Validation calls `sp_validate::run_checks()` directly (no subprocess). Institution registry loads from `CATALOG_PATH`. All 7 endpoints preserved with identical API contracts.
- **`scholarpress-cli`** gains `dump-spans` (word-level PDF extraction dump) and `dump-extract` (document-level extraction dump). Remains a thin wrapper over sp-extract and sp-validate.

## 4. API Contract (all 7 endpoints preserved)

### POST /extract
```
Request: multipart/form-data
  file: <binary>              PDF or DOCX
  ?institution: string        Optional

Response: 200 JSON
  { content: { pages, raw_text }, structure: { headings, front_matter, body, end_matter },
    metadata: { title, author, page_count, detected_fonts } }
Errors: 400 (unsupported format), 500 (extraction failure)
```

### POST /compile
```
Request: JSON
  { typst_code, variables? }
  ?institution: string        Required

Response: 200 PDF (application/pdf, raw bytes)
Errors: 500 (compilation failure)
```

### POST /validate
```
Request: JSON (50 MB body limit)
  { pdf_base64, institution }

Response: 200 JSON
  { violations: [{ check_id, status, detail, page }], pass_count, fail_count, error_count }
Errors: 404 (institution not found), 500 (validation failure)
```

### GET /health
```
Response: 200 text/plain  "ok"
```

### GET /institutions
```
Response: 200 JSON [{ id, name, ui_config? }]
```

### GET /institutions/:id/spec
```
Response: 200 JSON
  { id, yaml: "<raw>", summary: { document_structure, constants, automated_checks, human_checks } }
Errors: 404
```

### GET /institutions/:id/template
```
Response: 200 JSON
  { id, entry: "template.typ", files: [{ path, content }] }
Errors: 404
```

**Key change**: `/validate` no longer shells out to a CLI subprocess — calls `sp_validate::run_checks()` directly via the library.

## 5. Catalog Consumption

All consumers use **runtime loading via `CATALOG_PATH`** — not build-time embedding.

- Environment variable `CATALOG_PATH` points to the catalog repo
- Fallback: `../scholarpress-catalog/` sibling directory
- Docker: mounted as read-only volume

**Rationale**: Separates data release cycle from code release cycle. A YAML spec update should not trigger a Docker rebuild. Data can be updated instantly without touching application code. This pattern translates directly to Kubernetes (ConfigMaps, Persistent Volumes, Git-sync sidecars) for future cloud hosting.

## 6. CI/CD & Distribution

### GitHub Actions: `scholarpress-publish-ui`
- Trigger: push to main
- Build Next.js Dockerfile → push `ghcr.io/scholarpress-workshop/scholarpress-publish-ui:latest`

### GitHub Actions: `scholarpress-backend`
- Trigger: push to main
- Build publish-service Dockerfile from workspace root → push `ghcr.io/scholarpress-workshop/scholarpress-publish-service:latest`

CLI distributed via `cargo install` / pre-built binaries. No Docker image for CLI.

### `scholarpress-deliver` orchestration repo

```yaml
services:
  web:
    image: ghcr.io/scholarpress-workshop/scholarpress-publish-ui:latest
    ports:
      - "3000:3000"
    environment:
      - RUST_SERVICE_URL=http://rust-doc-service:4000
    depends_on:
      - rust-doc-service

  rust-doc-service:
    image: ghcr.io/scholarpress-workshop/scholarpress-publish-service:latest
    ports:
      - "4000:4000"
    environment:
      - CATALOG_PATH=/app/catalog
      - LLM_BASE_URL=https://reallms.rescloud.iu.edu/direct/v1
      - LLM_MODEL=gemma-4-31B-it
    volumes:
      - ../scholarpress-catalog:/app/catalog:ro
```

## 7. Migration Strategy

### Phase 1: Extract `scholarpress-publish-ui`
1. `git filter-repo --subdirectory-filter web/` on publish repo
2. Push to `scholarpress-workshop/scholarpress-publish-ui`
3. Add `AGENTS.md`, root `.gitignore`, verify `bun run build`
4. Add GitHub Actions workflow + Dockerfile

### Phase 2: Bootstrap `scholarpress-backend` workspace
1. Create `scholarpress-backend/` with top-level `Cargo.toml`
2. Move `scholarpress-check/src/` → `crates/sp-validate/src/` (preserve history)
3. Move `scholarpress-cli/src/` → `apps/scholarpress-cli/src/`
4. Create `crates/sp-extract/` from publish's rust-doc-service extraction code
5. Create `crates/sp-typst/` from publish's compile module + typst crate
6. Create `apps/publish-service/` from publish's rust-doc-service

### Phase 3: Unify extraction into `sp-extract`
1. Merge pdf_oxide logic from both codebases into `pdf.rs`
2. Merge DOCX parser from publish → `docx.rs`
3. Merge heading detector, chunker, document models
4. Expose `extract_pdf()` (rich, for publish) and `extract_pdf_spans()` (word-level, for validate)
5. Update `sp-validate` to depend on `sp-extract`

### Phase 4: Replace typst subprocess with `sp-typst`
1. Add `typst` + `typst-pdf` crates
2. Implement `compile(source, root?)` using native typst API
3. Move template loading/render into sp-typst
4. Update publish-service to call `sp_typst::compile()`

### Phase 5: Wire publish-service
1. Update config: `INSTITUTIONS_PATH` → `CATALOG_PATH`
2. Update institution registry to load from catalog
3. Replace validate subprocess with direct `sp_validate::run_checks()` call
4. Ensure all 7 endpoints work with identical API contracts
5. Multi-stage Dockerfile from workspace root

### Phase 6: CI/CD & verification
1. Add GitHub Actions workflows to both repos
2. Build and push GHCR images
3. Create `scholarpress-deliver` repo with docker-compose.yml
4. End-to-end test: `docker-compose up`, upload PDF, extract, compile, validate

## 8. Current State Reference
- `scholarpress-catalog`: 1 institution (IU), 41 checks, 16-section Typst template, 10 synthetic fixtures, 16 corpus PDFs (download.sh)
- `scholarpress-check`: Rust library crate, 36 checkers, 83 tests, 0 clippy warnings. PDF-only extraction via pdf_oxide. No main.rs (CLI extracted to scholarpress-cli). v0.1.0 on branch `main` at `4d93064` (old README) / `d4da72b` (current README).
- `scholarpress-publish`: Rust doc service (7 endpoints, axum, pdf_oxide + quick-xml extraction, typst subprocess compilation, diss-check subprocess validation) + Next.js chat frontend (AI SDK v7, shadcn/ui dark theme, Postgres). Dockerized via docker-compose. Still references `../../diss-check` and `INSTITUTIONS_PATH` (Phase 4 wiring pending). `1ad884c` on main with README.
- `scholarpress-cli`: Thin wrapper (check + calibrate subcommands) over scholarpress-check. 1 commit on `master`.
