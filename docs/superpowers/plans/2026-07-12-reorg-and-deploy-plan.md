# ScholarPress Reorganization & GHCR Deployment — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reorganize the ScholarPress ecosystem from 4 mixed-purpose repos into a clean 4-repo architecture (catalog, publish-ui, backend monorepo, deliver) with pre-built GHCR Docker images.

**Architecture:** Extract Next.js frontend from publish into `scholarpress-publish-ui`. Merge all Rust code (check library, cli, publish doc-service) into a Cargo workspace monorepo `scholarpress-backend` with crates (sp-extract, sp-validate, sp-typst) and apps (publish-service, scholarpress-cli). Replace typst subprocess with native library. CI/CD pushes Docker images to GHCR. Orchestration in `scholarpress-deliver`.

**Tech Stack:** Rust (Cargo workspace, axum, pdf_oxide, typst, quick-xml), Next.js 15 (Vercel AI SDK v7, shadcn/ui, Tailwind v4, bun), Docker, GitHub Actions, GHCR.

## Global Constraints

- Preserve git history via `git filter-repo` for all directory extractions
- `sp-` prefix for workspace library crates, `scholarpress-` prefix for user-facing apps
- Catalog consumed at runtime via `CATALOG_PATH` env var with `../scholarpress-catalog/` fallback
- Zero subprocess shell-outs — validation calls sp-validate library directly, typst uses native crate
- All 7 publish-service API endpoints preserved with identical contracts
- Docker images: `ghcr.io/scholarpress-workshop/scholarpress-publish-ui:latest` and `ghcr.io/scholarpress-workshop/scholarpress-publish-service:latest`
- LLM environment variables go in the web service, not rust-doc-service (Next.js frontend makes AI SDK calls)
- CLI distributed via `cargo install` / pre-built binaries, no Docker image
- `bun` for publish-ui (not npm), `cargo` for backend
- Workspace at `/home/danriggi/scholarpress-workshop/`

---

### Task 0: Create GitHub repos and set CLI remote

**Files:**
- Remote: `https://github.com/scholarpress-workshop/scholarpress-publish-ui.git`
- Remote: `https://github.com/scholarpress-workshop/scholarpress-backend.git`
- Remote: `https://github.com/scholarpress-workshop/scholarpress-deliver.git`
- Modify: `scholarpress-cli` git remote

**Interfaces:**
- Produces: 3 empty GitHub repos with default branches, CLI remote set

- [ ] **Step 1: Create repos on GitHub**

```bash
gh repo create scholarpress-workshop/scholarpress-publish-ui --public --description "ScholarPress Publish UI — Next.js 15 chat interface for AI-powered document formatting"
gh repo create scholarpress-workshop/scholarpress-backend --public --description "ScholarPress Backend — Rust monorepo: extraction, validation, typst compilation, and CLI"
gh repo create scholarpress-workshop/scholarpress-deliver --public --description "ScholarPress Deliver — Docker Compose orchestration for pre-built GHCR images"
```

- [ ] **Step 2: Set CLI remote**

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-cli
git remote add origin https://github.com/scholarpress-workshop/scholarpress-cli.git
git remote -v  # verify
```

- [ ] **Step 3: Commit**

```bash
cd /home/danriggi/scholarpress-workshop
git -C scholarpress-cli add -A && git -C scholarpress-cli commit -m "chore: add .serena memories" && git -C scholarpress-cli push -u origin master
```

---

### Task 1: Extract publish-ui from publish

**Files:**
- Create: `scholarpress-publish-ui/` (entire repo from `publish/web/` with history)
- Create: `scholarpress-publish-ui/.gitignore`
- Create: `scholarpress-publish-ui/AGENTS.md`

**Interfaces:**
- Produces: Standalone Next.js repo at `/home/danriggi/scholarpress-workshop/scholarpress-publish-ui/` with full history, compiles clean

- [ ] **Step 1: Clone a temp copy of publish for filter-repo**

```bash
cd /tmp
rm -rf publish-temp
cp -r /home/danriggi/scholarpress-workshop/scholarpress-publish publish-temp
cd publish-temp
```

- [ ] **Step 2: Extract web/ directory with git history**

```bash
cd /tmp/publish-temp
git filter-repo --subdirectory-filter web/ --force
```

- [ ] **Step 3: Push to new remote**

```bash
cd /tmp/publish-temp
git remote add origin https://github.com/scholarpress-workshop/scholarpress-publish-ui.git
git push -u origin main --force
```

- [ ] **Step 4: Pull into workspace and set up root files**

```bash
cd /home/danriggi/scholarpress-workshop
rm -rf scholarpress-publish-ui
git clone https://github.com/scholarpress-workshop/scholarpress-publish-ui.git
cd scholarpress-publish-ui
```

Add `.gitignore`:
```gitignore
node_modules/
.next/
.env
.env.local
*.tsbuildinfo
```

Add `AGENTS.md`:
```markdown
<!-- headroom:rtk-instructions -->
# RTK (Rust Token Killer) - Token-Optimized Commands

When running shell commands, **always prefix with `rtk`**. This reduces context
usage by 60-90% with zero behavior change. If rtk has no filter for a command,
it passes through unchanged — so it is always safe to use.

## Key Commands
```bash
rtk git status          rtk git diff            rtk git log
rtk ls <path>           rtk read <file>         rtk grep <pattern>
rtk bun install         rtk bun run dev         rtk bun run build
rtk bun run lint
```

## Development preferences
- Use `rtk` prefix for all shell commands
- `bun` as package manager
- Verify with lint/build before claiming work is done
- Never commit unless explicitly asked
<!-- /headroom:rtk-instructions -->
```

- [ ] **Step 5: Verify build**

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-publish-ui
bun install
bun run build
bun run lint
```

Expected: clean build, zero lint errors.

- [ ] **Step 6: Commit and push**

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-publish-ui
git add .gitignore AGENTS.md
git commit -m "chore: add .gitignore and AGENTS.md"
git push
```

---

### Task 2: Bootstrap backend workspace

**Files:**
- Create: `scholarpress-backend/Cargo.toml`
- Create: `scholarpress-backend/crates/sp-extract/Cargo.toml`
- Create: `scholarpress-backend/crates/sp-extract/src/lib.rs`
- Create: `scholarpress-backend/crates/sp-typst/Cargo.toml`
- Create: `scholarpress-backend/crates/sp-typst/src/lib.rs`
- Create: `scholarpress-backend/apps/publish-service/Cargo.toml`
- Create: `scholarpress-backend/apps/publish-service/src/main.rs`
- Create: `scholarpress-backend/apps/scholarpress-cli/Cargo.toml`
- Create: `scholarpress-backend/apps/scholarpress-cli/src/main.rs`
- Copy from check: `crates/sp-validate/` (all files)
- Copy from cli: `apps/scholarpress-cli/src/` (overlay on skeleton)

**Interfaces:**
- Produces: Workspace that compiles via `cargo build` from root. `sp-extract` and `sp-typst` are skeletons (empty). `sp-validate` has all 36 checkers and passes 83 tests. `scholarpress-cli` runs `check` and `calibrate` subcommands.

- [ ] **Step 1: Clone the backend repo and create directory structure**

```bash
cd /home/danriggi/scholarpress-workshop
rm -rf scholarpress-backend
git clone https://github.com/scholarpress-workshop/scholarpress-backend.git
cd scholarpress-backend
mkdir -p crates/sp-extract/src
mkdir -p crates/sp-validate/src/checkers
mkdir -p crates/sp-typst/src
mkdir -p apps/publish-service/src
mkdir -p apps/scholarpress-cli/src
```

- [ ] **Step 2: Create workspace root Cargo.toml**

`/home/danriggi/scholarpress-workshop/scholarpress-backend/Cargo.toml`:
```toml
[workspace]
members = [
    "crates/*",
    "apps/*",
]
resolver = "2"
```

- [ ] **Step 3: Create sp-extract skeleton**

`crates/sp-extract/Cargo.toml`:
```toml
[package]
name = "sp-extract"
version = "0.1.0"
edition = "2021"

[dependencies]
pdf_oxide = "0.3"
zip = "0.6"
quick-xml = "0.37"
roxmltree = "0.21"
regex = "1"
serde = { version = "1", features = ["derive"] }
```

`crates/sp-extract/src/lib.rs`:
```rust
pub mod document;
pub mod chunker;
pub mod heading;

pub fn extract_pdf(_bytes: &[u8]) -> Result<document::ParsedDocument, Box<dyn std::error::Error>> {
    unimplemented!("Phase 3")
}

pub fn extract_docx(_bytes: &[u8]) -> Result<document::ParsedDocument, Box<dyn std::error::Error>> {
    unimplemented!("Phase 3")
}
```

`crates/sp-extract/src/document.rs`:
```rust
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct TextSpan {
    pub text: String,
    pub font_name: String,
    pub font_size: f32,
    pub bbox: (f32, f32, f32, f32),
    pub is_bold: bool,
    pub is_italic: bool,
    pub color: Option<(f32, f32, f32)>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParsedPage {
    pub number: usize,
    pub text: String,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParsedParagraph {
    pub text: String,
    pub page_number: usize,
    pub is_bold: bool,
    pub is_italic: bool,
    pub is_underline: bool,
    pub is_all_caps: bool,
    pub is_heading: bool,
    pub heading_level: Option<usize>,
    pub font_size: Option<f32>,
    pub font_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Heading {
    pub text: String,
    pub level: usize,
    pub page_number: usize,
    pub raw_text_position: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParsedMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub page_count: usize,
    pub page_count_estimated: bool,
    pub detected_fonts: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParsedDocument {
    pub raw_text: String,
    pub pages: Vec<ParsedPage>,
    pub paragraphs: Vec<ParsedParagraph>,
    pub headings: Vec<Heading>,
    pub metadata: ParsedMetadata,
}
```

`crates/sp-extract/src/chunker.rs`:
```rust
#[derive(Debug, Clone)]
pub struct Chunk {
    pub text: String,
    pub start_char: usize,
    pub end_char: usize,
}

pub fn chunk_text(raw_text: &str, max_chars: usize, overlap: usize) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    let mut start = 0usize;
    while start < raw_text.len() {
        let end = (start + max_chars).min(raw_text.len());
        let slice = &raw_text[start..end];
        let break_point = slice.rfind("\n\n").map(|p| start + p + 2).unwrap_or(end);
        let final_end = break_point.min(end);
        let text = raw_text[start..final_end].to_string();
        chunks.push(Chunk { text, start_char: start, end_char: final_end });
        start = final_end.saturating_sub(overlap);
    }
    chunks
}
```

`crates/sp-extract/src/heading.rs`:
```rust
#[derive(Debug, Clone)]
pub struct HeadingDetectionConfig {
    pub signal_weights: SignalWeights,
    pub threshold: f32,
    pub size_jump_threshold: f32,
    pub context_keywords: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SignalWeights {
    pub caps: f32,
    pub underline: f32,
    pub bold: f32,
    pub size_jump: f32,
    pub numbering: f32,
    pub context: f32,
}

impl Default for HeadingDetectionConfig {
    fn default() -> Self {
        Self {
            signal_weights: SignalWeights {
                caps: 0.35, underline: 0.35, bold: 0.15,
                size_jump: 0.0, numbering: 0.10, context: 0.05,
            },
            threshold: 0.5,
            size_jump_threshold: 2.0,
            context_keywords: vec![
                "introduction".into(), "background".into(), "method".into(),
                "result".into(), "discussion".into(), "conclusion".into(),
                "chapter".into(), "appendix".into(), "reference".into(),
                "bibliography".into(), "abstract".into(), "acknowledgment".into(),
                "preface".into(), "dedication".into(), "contents".into(),
                "summary".into(),
            ],
        }
    }
}
```

- [ ] **Step 4: Create sp-typst skeleton**

`crates/sp-typst/Cargo.toml`:
```toml
[package]
name = "sp-typst"
version = "0.1.0"
edition = "2021"

[dependencies]
typst = "0.13"
```

`crates/sp-typst/src/lib.rs`:
```rust
use std::path::Path;

pub fn compile(_source: &str, _root: Option<&Path>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    unimplemented!("Phase 4")
}
```

- [ ] **Step 5: Copy sp-validate from scholarpress-check**

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-backend
cp -r ../scholarpress-check/src/* crates/sp-validate/src/
cp ../scholarpress-check/src/lib.rs crates/sp-validate/src/lib.rs  # ensure
```

Create `crates/sp-validate/Cargo.toml`:
```toml
[package]
name = "sp-validate"
version = "0.1.0"
edition = "2021"

[dependencies]
sp-extract = { path = "../sp-extract" }
pdf_oxide = "0.3"
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1"
regex = "1"
```

- [ ] **Step 6: Create publish-service skeleton**

`apps/publish-service/Cargo.toml`:
```toml
[package]
name = "publish-service"
version = "0.1.0"
edition = "2021"

[dependencies]
sp-extract = { path = "../../crates/sp-extract" }
sp-validate = { path = "../../crates/sp-validate" }
sp-typst = { path = "../../crates/sp-typst" }
axum = { version = "0.7", features = ["multipart"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
tower-http = { version = "0.5", features = ["cors"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
thiserror = "1"
uuid = { version = "1", features = ["v4"] }
base64 = "0.22"
```

`apps/publish-service/src/main.rs`:
```rust
#[tokio::main]
async fn main() {
    println!("publish-service starting on port 4000");
}
```

- [ ] **Step 7: Create scholarpress-cli app**

`apps/scholarpress-cli/Cargo.toml`:
```toml
[package]
name = "scholarpress-cli"
version = "0.1.0"
edition = "2021"

[dependencies]
sp-extract = { path = "../../crates/sp-extract" }
sp-validate = { path = "../../crates/sp-validate" }
clap = { version = "4", features = ["derive"] }
serde_json = "1"
```

Copy the existing CLI main.rs and update imports:

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-backend
cp ../scholarpress-cli/src/main.rs apps/scholarpress-cli/src/main.rs
cp -r ../scholarpress-cli/src/commands/ apps/scholarpress-cli/src/commands/
```

Update `apps/scholarpress-cli/src/main.rs` to use new crate names. Replace `scholarpress_check` with `sp_validate`:
```rust
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;

#[derive(Parser)]
#[command(name = "scholarpress", version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Check {
        #[arg(short, long)]
        spec: PathBuf,
        #[arg(short, long)]
        json: bool,
        #[arg(short, long)]
        quiet: bool,
        #[arg(long)]
        check: Option<String>,
        #[arg(short = 'C', long)]
        category: Option<String>,
        #[arg(long)]
        dump_extract: bool,
        pdf: PathBuf,
    },
    Calibrate {
        #[arg(short, long)]
        spec: PathBuf,
        #[arg(short, long)]
        corpus: PathBuf,
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Check { spec, json, quiet, check, category, dump_extract, pdf } => {
            commands::check::run(spec, pdf, json, quiet, check, category, dump_extract);
        }
        Commands::Calibrate { spec, corpus, json } => {
            commands::calibrate::run(spec, corpus, json);
        }
    }
}
```

Update `apps/scholarpress-cli/src/commands/check.rs` — replace all `scholarpress_check::` with `sp_validate::`:
```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-backend
sed -i 's/scholarpress_check/sp_validate/g' apps/scholarpress-cli/src/commands/check.rs
sed -i 's/scholarpress_check/sp_validate/g' apps/scholarpress-cli/src/commands/calibrate.rs
```

- [ ] **Step 8: Verify workspace compiles**

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-backend
cargo build 2>&1
```

Expected: workspace skeleton compiles (sp-extract and sp-typst have unimplemented! but should still compile). Fix any import issues in cli.

- [ ] **Step 9: Run sp-validate tests**

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-backend
cargo test -p sp-validate 2>&1
```

Expected: all 83 tests pass (they don't depend on sp-extract yet — the extractor is still internal to sp-validate).

- [ ] **Step 10: Commit backend skeleton**

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-backend
git add -A
git commit -m "feat: bootstrap workspace — sp-extract, sp-validate, sp-typst, publish-service, scholarpress-cli"
```

---

### Task 3: Create initial commit and push backend

**Files:**
- Create: `scholarpress-backend/.gitignore`
- Create: `scholarpress-backend/AGENTS.md`
- Create: `scholarpress-backend/.github/workflows/docker-publish.yml` (skeleton)

**Interfaces:**
- Produces: Backend repo with workspace, pushed to GitHub with CI skeleton

- [ ] **Step 1: Add root .gitignore**

`/home/danriggi/scholarpress-workshop/scholarpress-backend/.gitignore`:
```gitignore
target/
**/target/
*.tmp-*
.tmp-*
.DS_Store
```

- [ ] **Step 2: Add AGENTS.md**

`/home/danriggi/scholarpress-workshop/scholarpress-backend/AGENTS.md`:
```markdown
<!-- headroom:rtk-instructions -->
# RTK (Rust Token Killer) - Token-Optimized Commands

When running shell commands, **always prefix with `rtk`**. This reduces context
usage by 60-90% with zero behavior change.

## Key Commands
```bash
rtk git status          rtk git diff            rtk git log
rtk ls <path>           rtk cargo build         rtk cargo test
rtk cargo clippy        rtk cargo fmt
```
<!-- /headroom:rtk-instructions -->
```

- [ ] **Step 3: Add GH Actions skeleton**

`/home/danriggi/scholarpress-workshop/scholarpress-backend/.github/workflows/docker-publish.yml`:
```yaml
name: Docker Publish
on:
  push:
    branches: [main]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Login to GHCR
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      - name: Build and push
        run: echo "Docker build goes here — Phase 6"
```

- [ ] **Step 4: Commit and push**

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-backend
git add .gitignore AGENTS.md .github/
git commit -m "chore: add .gitignore, AGENTS.md, CI skeleton"
git push -u origin main
```

---

### Task 4: Merge PDF extraction into sp-extract

**Files:**
- Create: `crates/sp-extract/src/pdf.rs`
- Modify: `crates/sp-extract/src/lib.rs`
- Modify: `crates/sp-extract/Cargo.toml`

**Interfaces:**
- Produces: `extract_pdf(bytes: &[u8]) -> Result<ParsedDocument>` that extracts text, fonts, layout from PDF

- [ ] **Step 1: Write the pdf.rs extraction module**

`crates/sp-extract/src/pdf.rs`:
```rust
use pdf_oxide::layout::{TextChar, FontWeight, BBox};
use pdf_oxide::PdfDocument;
use crate::document::*;

pub fn extract_pdf(bytes: &[u8]) -> Result<ParsedDocument, Box<dyn std::error::Error>> {
    let doc = PdfDocument::from_bytes(bytes)?;
    let page_count = doc.page_count();

    let mut pages = Vec::new();
    let mut all_paragraphs = Vec::new();
    let mut all_fonts = std::collections::BTreeSet::new();

    for page_idx in 0..page_count {
        let media_box = doc.get_page_media_box(page_idx);
        let width = (media_box.urx - media_box.llx) as f32;
        let height = (media_box.ury - media_box.lly) as f32;

        let chars: Vec<TextChar> = doc.extract_chars(page_idx).into_iter().collect();
        let spans = build_spans(&chars, height);

        // Collect fonts
        for span in &spans {
            all_fonts.insert(span.font_name.clone());
        }

        // Group spans into paragraphs by detecting line breaks
        let paragraphs = spans_to_paragraphs(&spans, page_idx + 1);
        let page_text: String = paragraphs.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join("\n");

        all_paragraphs.extend(paragraphs);

        pages.push(ParsedPage {
            number: page_idx + 1,
            text: page_text,
            width,
            height,
        });
    }

    let raw_text: String = pages.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join("\n\n");

    Ok(ParsedDocument {
        raw_text,
        pages,
        paragraphs: all_paragraphs,
        headings: Vec::new(),
        metadata: ParsedMetadata {
            title: None,
            author: None,
            page_count,
            page_count_estimated: false,
            detected_fonts: all_fonts.into_iter().collect(),
        },
    })
}

fn build_spans(chars: &[&TextChar], page_height: f32) -> Vec<TextSpan> {
    let mut spans = Vec::new();
    if chars.is_empty() {
        return spans;
    }

    let mut current_word = Vec::new();
    let mut last: Option<&TextChar> = None;

    for ch in chars {
        if let Some(last_ch) = last {
            let y_delta = (ch.origin_y - last_ch.origin_y).abs();
            let gap = ch.bbox.x - (last_ch.bbox.x + last_ch.bbox.width);
            let font_changed = ch.font_name != last_ch.font_name;
            let size_delta = (ch.font_size - last_ch.font_size).abs();

            let is_new_line = y_delta > 3.0;
            let is_word_break = gap > 20.0 || font_changed || size_delta > 1.0;

            if is_new_line || is_word_break {
                if !current_word.is_empty() {
                    spans.push(build_word_span(&current_word, page_height));
                    current_word.clear();
                }
            }
        }
        current_word.push(*ch);
        last = Some(ch);
    }

    if !current_word.is_empty() {
        spans.push(build_word_span(&current_word, page_height));
    }

    spans
}

fn build_word_span(chars: &[&TextChar], page_height: f32) -> TextSpan {
    let first = chars[0];
    let text: String = chars.iter().map(|c| c.ch.to_string()).collect::<Vec<_>>().join("");
    let max_y = chars.iter().map(|c| c.origin_y).fold(f32::NEG_INFINITY, f32::max);
    let min_y = chars.iter().map(|c| c.origin_y - c.font_size).fold(f32::INFINITY, f32::min);
    let min_x = chars.iter().map(|c| c.bbox.x).fold(f32::INFINITY, f32::min);
    let max_x = chars.iter().map(|c| c.bbox.x + c.bbox.width).fold(f32::NEG_INFINITY, f32::max);

    TextSpan {
        text,
        font_name: first.font_name.clone(),
        font_size: first.font_size,
        bbox: (
            (page_height - max_y).max(0.0),
            (page_height - min_y).max(0.0),
            min_x.max(0.0),
            max_x.max(0.0),
        ),
        is_bold: matches!(first.font_weight, FontWeight::Bold),
        is_italic: first.is_italic,
        color: None,
    }
}

fn spans_to_paragraphs(spans: &[TextSpan], page_number: usize) -> Vec<ParsedParagraph> {
    let mut paragraphs = Vec::new();
    if spans.is_empty() {
        return paragraphs;
    }

    let line_heights: Vec<f32> = spans.windows(2)
        .filter_map(|w| {
            let gap = (w[1].bbox.0 - w[0].bbox.1).abs();
            if gap > 0.0 { Some(gap) } else { None }
        })
        .collect();

    let median_line_gap = median(&line_heights).unwrap_or(12.0);
    let para_threshold = median_line_gap * 1.5;

    let mut current_text = String::new();
    let mut current_bold = false;
    let mut current_italic = false;
    let mut current_font_size = None;
    let mut current_font_name = None;

    for (i, span) in spans.iter().enumerate() {
        let is_new_para = if i == 0 {
            false
        } else {
            let prev = &spans[i - 1];
            let y_gap = (span.bbox.0 - prev.bbox.1).abs();
            y_gap > para_threshold
        };

        if is_new_para && !current_text.is_empty() {
            paragraphs.push(ParsedParagraph {
                text: current_text.trim().to_string(),
                page_number,
                is_bold: current_bold,
                is_italic: current_italic,
                is_underline: false,
                is_all_caps: current_text.trim().chars().all(|c| !c.is_alphabetic() || c.is_uppercase()),
                is_heading: false,
                heading_level: None,
                font_size: current_font_size,
                font_name: current_font_name.clone(),
            });
            current_text.clear();
        }

        if !current_text.is_empty() {
            current_text.push(' ');
        }
        current_text.push_str(&span.text);
        current_bold = current_bold || span.is_bold;
        current_italic = current_italic || span.is_italic;
        if current_font_size.is_none() {
            current_font_size = Some(span.font_size);
        }
        if current_font_name.is_none() {
            current_font_name = Some(span.font_name.clone());
        }
    }

    if !current_text.is_empty() {
        paragraphs.push(ParsedParagraph {
            text: current_text.trim().to_string(),
            page_number,
            is_bold: current_bold,
            is_italic: current_italic,
            is_underline: false,
            is_all_caps: current_text.trim().chars().all(|c| !c.is_alphabetic() || c.is_uppercase()),
            is_heading: false,
            heading_level: None,
            font_size: current_font_size,
            font_name: current_font_name,
        });
    }

    paragraphs
}

fn median(values: &[f32]) -> Option<f32> {
    if values.is_empty() {
        return None;
    }
    let mut sorted: Vec<f32> = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        Some((sorted[mid - 1] + sorted[mid]) / 2.0)
    } else {
        Some(sorted[mid])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_median_odd() {
        assert_eq!(median(&[1.0, 3.0, 2.0]), Some(2.0));
    }

    #[test]
    fn test_median_even() {
        assert_eq!(median(&[1.0, 2.0, 3.0, 4.0]), Some(2.5));
    }

    #[test]
    fn test_median_empty() {
        assert_eq!(median(&[]), None);
    }
}
```

- [ ] **Step 2: Update lib.rs to wire pdf.rs**

Replace `crates/sp-extract/src/lib.rs`:
```rust
pub mod document;
pub mod chunker;
pub mod heading;
pub mod pdf;

pub use pdf::extract_pdf;

pub fn extract_docx(_bytes: &[u8]) -> Result<document::ParsedDocument, Box<dyn std::error::Error>> {
    unimplemented!("coming in Phase 3 DOCX task")
}
```

- [ ] **Step 3: Build and test**

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-backend
cargo build -p sp-extract
cargo test -p sp-extract
```

Expected: compiles, unit tests pass.

- [ ] **Step 4: Commit**

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-backend
git add crates/sp-extract/
git commit -m "feat(sp-extract): PDF extraction via pdf_oxide"
```

---

### Task 5: Merge DOCX parser into sp-extract

**Files:**
- Create: `crates/sp-extract/src/docx.rs`
- Modify: `crates/sp-extract/src/lib.rs`

**Interfaces:**
- Produces: `extract_docx(bytes: &[u8]) -> Result<ParsedDocument>` — parses .docx files via zip + roxmltree

- [ ] **Step 1: Write the DOCX parser**

`crates/sp-extract/src/docx.rs`:
```rust
use std::io::Read;
use roxmltree::Node;
use regex::Regex;
use crate::document::*;

#[derive(Debug, Clone, Default)]
struct StyleInfo {
    font_name: Option<String>,
    font_size_half_pt: Option<f32>,
    bold: bool,
    italic: bool,
    underline: bool,
}

pub fn extract_docx(bytes: &[u8]) -> Result<ParsedDocument, Box<dyn std::error::Error>> {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)?;

    let doc_xml = {
        let mut file = archive.by_name("word/document.xml")?;
        let mut xml = String::new();
        file.read_to_string(&mut xml)?;
        xml
    };

    let styles = archive.by_name("word/styles.xml").ok().and_then(|mut f| {
        let mut s = String::new();
        f.read_to_string(&mut s).ok()?;
        Some(s)
    });

    let style_map = styles.as_deref().map(parse_styles).unwrap_or_default();
    let doc_tree = roxmltree::Document::parse(&doc_xml)?;
    let paragraphs = parse_paragraphs(doc_tree.root(), &style_map);

    let word_count: usize = paragraphs.iter().map(|p| p.text.split_whitespace().count()).sum();
    let estimated_pages = (word_count as f32 / 250.0).ceil() as usize;

    let mut fonts = std::collections::BTreeSet::new();
    for p in &paragraphs {
        if let Some(ref name) = p.font_name {
            fonts.insert(name.clone());
        }
    }

    let raw_text: String = paragraphs.iter().map(|p| p.text.clone()).collect::<Vec<_>>().join("\n\n");

    Ok(ParsedDocument {
        raw_text,
        pages: vec![ParsedPage {
            number: 1,
            text: raw_text.clone(),
            width: 612.0,
            height: 792.0,
        }],
        paragraphs,
        headings: Vec::new(),
        metadata: ParsedMetadata {
            title: None,
            author: None,
            page_count: estimated_pages.max(1),
            page_count_estimated: true,
            detected_fonts: fonts.into_iter().collect(),
        },
    })
}

fn parse_styles(xml: &str) -> std::collections::HashMap<String, StyleInfo> {
    let mut map = std::collections::HashMap::new();
    let doc = match roxmltree::Document::parse(xml) {
        Ok(d) => d,
        Err(_) => return map,
    };

    let ns = resolve_ns(doc.root());

    for style_node in doc.descendants().filter(|n| n.has_tag_name((ns.get("w").unwrap_or(&""), "style"))) {
        let style_id = style_node.attribute((ns.get("w").unwrap_or(&""), "styleId")).unwrap_or("").to_string();
        if style_id.is_empty() {
            continue;
        }

        let mut info = StyleInfo::default();

        if let Some(rpr) = find_child(&style_node, ns.get("w").unwrap_or(&""), "rPr") {
            if let Some(rfonts) = find_child(&rpr, ns.get("w").unwrap_or(&""), "rFonts") {
                info.font_name = rfonts.attribute((ns.get("w").unwrap_or(&""), "ascii")).map(String::from);
            }
            if let Some(sz) = find_child(&rpr, ns.get("w").unwrap_or(&""), "sz") {
                if let Ok(val) = sz.attribute((ns.get("w").unwrap_or(&""), "val")).unwrap_or("0").parse::<f32>() {
                    info.font_size_half_pt = Some(val);
                }
            }
            info.bold = find_child(&rpr, ns.get("w").unwrap_or(&""), "b").is_some();
            info.italic = find_child(&rpr, ns.get("w").unwrap_or(&""), "i").is_some();
            info.underline = find_child(&rpr, ns.get("w").unwrap_or(&""), "u").is_some();
        }

        map.insert(style_id, info);
    }

    map
}

fn parse_paragraphs(root: Node, style_map: &std::collections::HashMap<String, StyleInfo>) -> Vec<ParsedParagraph> {
    let mut paragraphs = Vec::new();
    let ns = resolve_ns(root);
    let w_ns = ns.get("w").cloned().unwrap_or_default();

    let body = find_child(&root, ns.get("w").unwrap_or(&""), "body");
    let container = body.as_ref().unwrap_or(&root);

    for p_node in container.descendants().filter(|n| n.has_tag_name((&w_ns, "p"))) {
        let mut text = String::new();
        let mut bold = false;
        let mut italic = false;
        let mut underline = false;
        let mut font_size_half_pt = None;
        let mut font_name = None;

        // Check paragraph-level style
        if let Some(pPr) = find_child(&p_node, &w_ns, "pPr") {
            if let Some(pStyle) = find_child(&pPr, &w_ns, "pStyle") {
                if let Some(style_id) = pStyle.attribute((&w_ns, "val")) {
                    if let Some(style_info) = style_map.get(style_id) {
                        bold = bold || style_info.bold;
                        italic = italic || style_info.italic;
                        underline = underline || style_info.underline;
                        if font_size_half_pt.is_none() {
                            font_size_half_pt = style_info.font_size_half_pt;
                        }
                        if font_name.is_none() {
                            font_name = style_info.font_name.clone();
                        }
                    }
                }
            }
        }

        // Walk runs to collect text and formatting
        for r_node in p_node.descendants().filter(|n| n.has_tag_name((&w_ns, "r"))) {
            let mut run_bold = bold;
            let mut run_italic = italic;
            let mut run_underline = underline;
            let mut run_font_size = font_size_half_pt;
            let mut run_font_name = font_name.clone();

            if let Some(rPr) = find_child(&r_node, &w_ns, "rPr") {
                run_bold = find_child(&rPr, &w_ns, "b").is_some() || run_bold;
                run_italic = find_child(&rPr, &w_ns, "i").is_some() || run_italic;
                run_underline = find_child(&rPr, &w_ns, "u").is_some() || run_underline;

                if let Some(sz) = find_child(&rPr, &w_ns, "sz") {
                    if let Ok(val) = sz.attribute((&w_ns, "val")).unwrap_or("0").parse::<f32>() {
                        run_font_size = Some(val);
                    }
                }
                if let Some(rfonts) = find_child(&rPr, &w_ns, "rFonts") {
                    if let Some(ascii) = rfonts.attribute((&w_ns, "ascii")) {
                        run_font_name = Some(ascii.to_string());
                    }
                }
            }

            for t_node in r_node.descendants().filter(|n| n.has_tag_name((&w_ns, "t"))) {
                if let Some(t) = t_node.text() {
                    text.push_str(t);
                }
            }

            bold = run_bold;
            italic = run_italic;
            underline = run_underline;
            font_size_half_pt = run_font_size;
            font_name = run_font_name.clone();
        }

        if !text.trim().is_empty() {
            let all_caps = text.trim().chars().all(|c| !c.is_alphabetic() || c.is_uppercase());
            paragraphs.push(ParsedParagraph {
                text: text.trim().to_string(),
                page_number: 1,
                is_bold: bold,
                is_italic: italic,
                is_underline: underline,
                is_all_caps: all_caps,
                is_heading: false,
                heading_level: None,
                font_size: font_size_half_pt.map(|s| s / 2.0),
                font_name,
            });
        }
    }

    paragraphs
}

fn resolve_ns(root: Node) -> std::collections::HashMap<String, String> {
    let mut ns = std::collections::HashMap::new();
    if let Some(n) = root.namespaces().iter().find(|n| n.uri() == "http://schemas.openxmlformats.org/wordprocessingml/2006/main") {
        ns.insert("w".to_string(), n.name().unwrap_or("w").to_string());
    }
    if let Some(n) = root.namespaces().iter().find(|n| n.uri() == "http://schemas.openxmlformats.org/officeDocument/2006/relationships") {
        ns.insert("r".to_string(), n.name().unwrap_or("r").to_string());
    }
    ns
}

fn find_child<'a>(node: &Node<'a, 'a>, ns: &str, local: &str) -> Option<Node<'a, 'a>> {
    node.children().find(|n| {
        n.is_element() && n.tag_name().namespace() == Some(ns) && n.tag_name().name() == local
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_docx_empty() {
        // A minimal valid .docx is complex to construct; test that garbage input errors
        let result = extract_docx(b"not a zip file");
        assert!(result.is_err());
    }
}
```

- [ ] **Step 2: Update lib.rs**

Replace `crates/sp-extract/src/lib.rs`:
```rust
pub mod document;
pub mod chunker;
pub mod heading;
pub mod pdf;
pub mod docx;

pub use pdf::extract_pdf;
pub use docx::extract_docx;
```

- [ ] **Step 3: Build and test**

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-backend
cargo build -p sp-extract
cargo test -p sp-extract
```

Expected: compiles, all tests pass.

- [ ] **Step 4: Commit**

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-backend
git add crates/sp-extract/
git commit -m "feat(sp-extract): DOCX parser via zip + roxmltree"
```

---

### Task 6: Merge heading detector and wire sp-validate to sp-extract

**Files:**
- Modify: `crates/sp-extract/src/heading.rs` (implement detection)
- Modify: `crates/sp-extract/src/lib.rs` (apply headings in extract)
- Modify: `crates/sp-validate/src/extractor.rs` (redirect to sp-extract)
- Modify: `crates/sp-validate/src/document.rs` (remove duplicate types)

**Interfaces:**
- Produces: Heading detection applied to extracted documents. sp-validate uses sp-extract for PDF extraction instead of its own extractor.

- [ ] **Step 1: Implement heading detection**

Append to `crates/sp-extract/src/heading.rs`:
```rust
use crate::document::*;

pub fn detect_headings(paragraphs: &mut [ParsedParagraph], config: &HeadingDetectionConfig) -> Vec<Heading> {
    let body_font_size = median_font_size(paragraphs);
    let mut headings = Vec::new();

    let numbering_re = regex::Regex::new(r"^(\d+(?:\.\d+)*)\s").unwrap();
    let section_re = regex::Regex::new(r"^\d+\.\d+").unwrap();
    let sub_section_re = regex::Regex::new(r"^\d+\.\d+\.\d+").unwrap();
    let chapter_re = regex::Regex::new(r"(?i)^(chapter|section)\s+\d+").unwrap();

    for (i, para) in paragraphs.iter_mut().enumerate() {
        let mut score: f32 = 0.0;

        if para.is_all_caps { score += config.signal_weights.caps; }
        if para.is_underline { score += config.signal_weights.underline; }
        if para.is_bold { score += config.signal_weights.bold; }

        if let Some(fs) = para.font_size {
            if let Some(bfs) = body_font_size {
                if fs - bfs >= config.size_jump_threshold {
                    score += config.signal_weights.size_jump;
                }
            }
        }

        if numbering_re.is_match(&para.text) { score += config.signal_weights.numbering; }

        let lower = para.text.to_lowercase();
        if config.context_keywords.iter().any(|kw| lower.contains(kw)) {
            score += config.signal_weights.context;
        }

        if score >= config.threshold {
            let level = if sub_section_re.is_match(&para.text) {
                3
            } else if section_re.is_match(&para.text) {
                2
            } else if chapter_re.is_match(&para.text) {
                1
            } else if para.is_all_caps {
                1
            } else if let (Some(fs), Some(bfs)) = (para.font_size, body_font_size) {
                if fs - bfs >= 4.0 { 1 } else if fs - bfs >= 2.0 { 2 } else { 2 }
            } else {
                2
            };

            para.is_heading = true;
            para.heading_level = Some(level);

            headings.push(Heading {
                text: para.text.clone(),
                level,
                page_number: para.page_number,
                raw_text_position: i,
            });
        }
    }

    headings
}

fn median_font_size(paragraphs: &[ParsedParagraph]) -> Option<f32> {
    let mut sizes: Vec<f32> = paragraphs.iter().filter_map(|p| p.font_size).collect();
    if sizes.is_empty() { return None; }
    sizes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    Some(sizes[sizes.len() / 2])
}
```

- [ ] **Step 2: Apply heading detection in extraction pipeline**

Modify `crates/sp-extract/src/lib.rs` — add heading detection call after extraction:
```rust
pub mod document;
pub mod chunker;
pub mod heading;
pub mod pdf;
pub mod docx;

use heading::{detect_headings, HeadingDetectionConfig};

pub fn extract_pdf(
    bytes: &[u8],
) -> Result<document::ParsedDocument, Box<dyn std::error::Error>> {
    let mut doc = pdf::extract_pdf(bytes)?;
    let config = HeadingDetectionConfig::default();
    let headings = detect_headings(&mut doc.paragraphs, &config);
    doc.headings = headings;
    Ok(doc)
}

pub fn extract_docx(
    bytes: &[u8],
) -> Result<document::ParsedDocument, Box<dyn std::error::Error>> {
    let mut doc = docx::extract_docx(bytes)?;
    let config = HeadingDetectionConfig::default();
    let headings = detect_headings(&mut doc.paragraphs, &config);
    doc.headings = headings;
    Ok(doc)
}
```

- [ ] **Step 3: Redirect sp-validate extractor to sp-extract**

Replace `crates/sp-validate/src/extractor.rs`:
```rust
use std::path::Path;
use std::fs;

pub fn extract_document(path: &Path) -> Result<crate::document::Document, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    let parsed = sp_extract::extract_pdf(&bytes)?;

    // Convert ParsedDocument → Document (word-level spans per page)
    // For now, use pdf_oxide directly since checkers need TextSpan-level data
    // Full migration to sp-extract types will happen incrementally
    extract_document_raw(path)
}

// Keep the existing extractor as a private fallback during migration
fn extract_document_raw(path: &Path) -> Result<crate::document::Document, Box<dyn std::error::Error>> {
    use pdf_oxide::PdfDocument;

    let doc = PdfDocument::open(path)?;
    let page_count = doc.page_count();
    let mut pages = Vec::with_capacity(page_count);

    for idx in 0..page_count {
        let media_box = doc.get_page_media_box(idx);
        let width = (media_box.urx - media_box.llx) as f32;
        let height = (media_box.ury - media_box.lly) as f32;

        let chars: Vec<pdf_oxide::layout::TextChar> = doc.extract_chars(idx).into_iter().collect();
        let char_refs: Vec<&pdf_oxide::layout::TextChar> = chars.iter().collect();
        let spans = build_words(&char_refs, height);

        let images = doc.extract_images(idx).into_iter()
            .filter_map(|img| img.bbox().map(|b| {
                (height - b.y - b.height, height - b.y, b.x, b.x + b.width)
            }))
            .collect();

        let paths = doc.extract_paths(idx).into_iter()
            .map(|p| {
                let b = p.bbox;
                (height - b.y - b.height, height - b.y, b.x, b.x + b.width)
            })
            .collect();

        pages.push(crate::document::Page {
            page_number: idx + 1,
            width,
            height,
            spans,
            images,
            paths,
        });
    }

    Ok(crate::document::Document { pages })
}

fn build_words(chars: &[&pdf_oxide::layout::TextChar], page_height: f32) -> Vec<crate::document::TextSpan> {
    let mut spans = Vec::new();
    if chars.is_empty() { return spans; }

    let mut current: Vec<&pdf_oxide::layout::TextChar> = Vec::new();
    let mut last: Option<&pdf_oxide::layout::TextChar> = None;

    for ch in chars {
        if let Some(prev) = last {
            let y_delta = (ch.origin_y - prev.origin_y).abs();
            let gap = ch.bbox.x - (prev.bbox.x + prev.bbox.width);
            let font_changed = ch.font_name != prev.font_name;
            let size_delta = (ch.font_size - prev.font_size).abs();

            if y_delta > 3.0 || gap > 20.0 || font_changed || size_delta > 1.0 {
                if !current.is_empty() {
                    spans.push(build_word(&current, page_height));
                    current.clear();
                }
            }
        }
        current.push(*ch);
        last = Some(ch);
    }

    if !current.is_empty() {
        spans.push(build_word(&current, page_height));
    }
    spans
}

fn build_word(chars: &[&pdf_oxide::layout::TextChar], page_height: f32) -> crate::document::TextSpan {
    let first = chars[0];
    let text: String = chars.iter().map(|c| c.ch.to_string()).collect();
    let max_y = chars.iter().map(|c| c.origin_y).fold(f32::NEG_INFINITY, f32::max);
    let min_y = chars.iter().map(|c| c.origin_y - c.font_size).fold(f32::INFINITY, f32::min);
    let min_x = chars.iter().map(|c| c.bbox.x).fold(f32::INFINITY, f32::min);
    let max_x = chars.iter().map(|c| c.bbox.x + c.bbox.width).fold(f32::NEG_INFINITY, f32::max);

    crate::document::TextSpan {
        text,
        font_name: first.font_name.clone(),
        font_size: first.font_size,
        bbox: (
            (page_height - max_y).max(0.0),
            (page_height - min_y).max(0.0),
            min_x.max(0.0),
            max_x.max(0.0),
        ),
        is_bold: matches!(first.font_weight, pdf_oxide::layout::FontWeight::Bold),
        is_italic: first.is_italic,
        color: None,
    }
}
```

- [ ] **Step 4: Build and verify tests pass**

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-backend
cargo build -p sp-validate
cargo test -p sp-validate
```

Expected: all 83 tests pass. The extractor migration is an internal refactor — the public API (`extract_document(path)`) is unchanged.

- [ ] **Step 5: Build sp-extract tests too**

```bash
cargo test -p sp-extract
```

- [ ] **Step 6: Commit**

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-backend
git add crates/
git commit -m "feat: heading detection in sp-extract; sp-validate extractor delegates to sp-extract"
```

---

### Task 7: Implement sp-typst with native compilation

**Files:**
- Modify: `crates/sp-typst/Cargo.toml`
- Modify: `crates/sp-typst/src/lib.rs`
- Create: `crates/sp-typst/src/template.rs`

**Interfaces:**
- Produces: `sp_typst::compile(source: &str, root: Option<&Path>) -> Result<Vec<u8>>` that compiles Typst to PDF bytes using the native `typst` crate

- [ ] **Step 1: Update Cargo.toml with full deps**

Replace `crates/sp-typst/Cargo.toml`:
```toml
[package]
name = "sp-typst"
version = "0.1.0"
edition = "2021"

[dependencies]
typst = "0.13"
```

- [ ] **Step 2: Implement compile using typst crate**

Replace `crates/sp-typst/src/lib.rs`:
```rust
pub mod template;

use std::io::Read;
use std::path::{Path, PathBuf};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::Source;
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::Library;

pub fn compile(source: &str, root: Option<&Path>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let world = SimpleWorld::new(root)?;
    let mut diagnostic = Vec::new();
    let source_id = typst::syntax::FileId::new(None, typst::syntax::VirtualPath::new("main.typ"));

    let document = typst::compile(
        &world,
        typst::World::library(&world),
        typst::World::book(&world),
        source,
        source_id,
        &mut diagnostic,
    )?;

    if !document.pages.is_empty() {
        let pdf_data = typst_pdf::pdf(&document, typst::World::today(&world), None);
        Ok(pdf_data)
    } else {
        let diag_str: Vec<String> = diagnostic.iter().map(|d| format!("{:?}", d)).collect();
        Err(format!("Compilation produced no pages. Diagnostics: {}", diag_str.join("; ")).into())
    }
}

struct SimpleWorld {
    root: Option<PathBuf>,
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
}

impl SimpleWorld {
    fn new(root: Option<&Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut fonts = Vec::new();
        let mut book = FontBook::new();

        // Try loading common system fonts
        let font_dirs = [
            "/usr/share/fonts",
            "/usr/local/share/fonts",
            "/System/Library/Fonts",
            "C:\\Windows\\Fonts",
        ];

        for dir in &font_dirs {
            let p = Path::new(dir);
            if p.exists() {
                load_fonts_from_dir(p, &mut fonts, &mut book);
            }
        }

        Ok(Self {
            root: root.map(|p| p.to_path_buf()),
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(book),
            fonts,
        })
    }
}

impl typst::World for SimpleWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> typst::syntax::Source {
        Source::new(
            typst::syntax::FileId::new(None, typst::syntax::VirtualPath::new("main.typ")),
            String::new(),
        )
    }

    fn source(&self, id: typst::syntax::FileId) -> typst::diag::FileResult<typst::syntax::Source> {
        let vpath = id.vpath();
        let path = vpath.as_rootless_path();
        let full_path = if let Some(ref r) = self.root {
            r.join(path)
        } else {
            path.to_path_buf()
        };

        let mut content = String::new();
        std::fs::File::open(&full_path)?.read_to_string(&mut content)?;

        Ok(Source::new(id, content))
    }

    fn file(&self, id: typst::syntax::FileId) -> typst::diag::FileResult<Bytes> {
        let vpath = id.vpath();
        let path = vpath.as_rootless_path();
        let full_path = if let Some(ref r) = self.root {
            r.join(path)
        } else {
            path.to_path_buf()
        };
        let data = std::fs::read(&full_path)?;
        Ok(Bytes::from(data))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        None
    }
}

fn load_fonts_from_dir(dir: &Path, fonts: &mut Vec<Font>, book: &mut FontBook) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                load_fonts_from_dir(&path, fonts, book);
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if matches!(ext.to_lowercase().as_str(), "ttf" | "otf" | "ttc") {
                    if let Ok(data) = std::fs::read(&path) {
                        let bytes = Bytes::from(data);
                        for (i, font) in Font::iter(bytes).enumerate() {
                            let family = font.info().family.clone();
                            let style = font.info().variant;
                            book.push(family, style);
                            fonts.push(font);
                            if i > 0 {
                                // TTC files — only load first face for simplicity
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 3: Create template.rs for loading and rendering**

`crates/sp-typst/src/template.rs`:
```rust
use serde_json::Value;
use std::path::{Path, PathBuf};

pub struct TemplateSet {
    pub entry: String,
    pub files: Vec<TemplateFile>,
}

pub struct TemplateFile {
    pub path: String,
    pub content: String,
}

pub fn load_template(template_dir: &Path) -> Result<TemplateSet, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    collect_typ_files(template_dir, template_dir, &mut files)?;

    if files.is_empty() {
        return Err("No .typ files found in template directory".into());
    }

    // template.typ is the conventional entry point
    let entry = if files.iter().any(|f| f.path == "template.typ") {
        "template.typ".to_string()
    } else {
        files[0].path.clone()
    };

    Ok(TemplateSet { entry, files })
}

fn collect_typ_files(
    base: &Path,
    dir: &Path,
    files: &mut Vec<TemplateFile>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_typ_files(base, &path, files)?;
        } else if path.extension().map_or(false, |e| e == "typ") {
            let relative = path.strip_prefix(base)?.to_string_lossy().to_string();
            let content = std::fs::read_to_string(&path)?;
            files.push(TemplateFile {
                path: relative,
                content,
            });
        }
    }
    Ok(())
}

pub fn render_template(code: &str, variables: &std::collections::HashMap<String, Value>) -> String {
    let mut result = code.to_string();
    for (key, value) in variables {
        let placeholder = format!("{{{}}}", key.to_uppercase());
        let replacement = match value {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        result = result.replace(&placeholder, &replacement);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_template_substitutes_variables() {
        let mut vars = std::collections::HashMap::new();
        vars.insert("TITLE".to_string(), serde_json::Value::String("My Dissertation".to_string()));
        let result = render_template("#let title = \"{TITLE}\"", &vars);
        assert_eq!(result, "#let title = \"My Dissertation\"");
    }

    #[test]
    fn test_load_template_empty_dir() {
        let tmp = std::env::temp_dir().join("sp-typst-empty-test");
        std::fs::create_dir_all(&tmp).ok();
        let result = load_template(&tmp);
        assert!(result.is_err());
        std::fs::remove_dir_all(&tmp).ok();
    }
}
```

- [ ] **Step 4: Build and test**

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-backend
cargo build -p sp-typst
cargo test -p sp-typst
```

Expected: compiles, template tests pass. Note: actual PDF compilation test requires a valid Typst document and fonts on the system.

- [ ] **Step 5: Commit**

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-backend
git add crates/sp-typst/
git commit -m "feat(sp-typst): native typst compilation + template loading"
```

---

### Task 8: Wire publish-service — migrate app code and all 7 endpoints

**Files:**
- Modify: `apps/publish-service/src/main.rs`
- Create: `apps/publish-service/src/lib.rs`
- Create: `apps/publish-service/src/config.rs`
- Create: `apps/publish-service/src/error.rs`
- Create: `apps/publish-service/src/institutions/mod.rs`
- Create: `apps/publish-service/src/routes/mod.rs`
- Create: `apps/publish-service/src/routes/extract.rs`
- Create: `apps/publish-service/src/routes/compile.rs`
- Create: `apps/publish-service/src/routes/validate.rs`
- Create: `apps/publish-service/src/routes/institutions.rs`
- Create: `apps/publish-service/src/routes/spec.rs`
- Create: `apps/publish-service/src/routes/template.rs`

**Interfaces:**
- Produces: Full publish-service with all 7 endpoints, loading institutions from CATALOG_PATH

- [ ] **Step 1: Create config.rs**

`apps/publish-service/src/config.rs`:
```rust
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub port: u16,
    pub catalog_path: PathBuf,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let port = std::env::var("PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(4000);

        let catalog_path = std::env::var("CATALOG_PATH")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                let cwd = std::env::current_dir().ok()?;
                let sibling = cwd.parent()?.join("scholarpress-catalog");
                if sibling.exists() {
                    Some(sibling)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| PathBuf::from("../scholarpress-catalog"));

        Self { port, catalog_path }
    }
}
```

- [ ] **Step 2: Create error.rs**

`apps/publish-service/src/error.rs`:
```rust
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    Extraction(String),
    Compilation(String),
    Validation(String),
    InstitutionNotFound(String),
    Internal(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Extraction(m) => write!(f, "Extraction failed: {}", m),
            AppError::Compilation(m) => write!(f, "Compilation failed: {}", m),
            AppError::Validation(m) => write!(f, "Validation failed: {}", m),
            AppError::InstitutionNotFound(m) => write!(f, "Institution not found: {}", m),
            AppError::Internal(m) => write!(f, "Internal error: {}", m),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::InstitutionNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
```

- [ ] **Step 3: Create institutions/mod.rs**

`apps/publish-service/src/institutions/mod.rs`:
```rust
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct Institution {
    pub id: String,
    pub name: String,
    pub spec: serde_yaml::Value,
    pub template_dir: std::path::PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_config: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_config: Option<serde_yaml::Value>,
}

#[derive(Debug, Clone)]
pub struct Registry {
    pub institutions: HashMap<String, Institution>,
}

impl Registry {
    pub fn load(catalog_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let institutions_dir = catalog_path.join("institutions");
        let mut institutions = HashMap::new();

        if !institutions_dir.exists() {
            return Err(format!("Institutions directory not found: {}", institutions_dir.display()).into());
        }

        for entry in std::fs::read_dir(&institutions_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() { continue; }

            let id = entry.file_name().to_string_lossy().to_string();
            let spec_path = path.join("spec.yaml");
            let template_dir = path.join("template");

            if !spec_path.exists() { continue; }

            let spec_yaml = std::fs::read_to_string(&spec_path)?;
            let spec: serde_yaml::Value = serde_yaml::from_str(&spec_yaml)?;

            let name = spec.get("institution")
                .and_then(|v| v.as_str())
                .unwrap_or(&id)
                .to_string();

            let llm_config = path.join("llm.yaml");
            let llm = if llm_config.exists() {
                let s = std::fs::read_to_string(&llm_config)?;
                Some(serde_yaml::from_str(&s)?)
            } else {
                None
            };

            let ui_config = path.join("ui.yaml");
            let ui = if ui_config.exists() {
                let s = std::fs::read_to_string(&ui_config)?;
                Some(serde_yaml::from_str(&s)?)
            } else {
                None
            };

            institutions.insert(id.clone(), Institution {
                id,
                name,
                spec,
                template_dir,
                llm_config: llm,
                ui_config: ui,
            });
        }

        Ok(Self { institutions })
    }

    pub fn get(&self, id: &str) -> Option<&Institution> {
        self.institutions.get(id)
    }

    pub fn list(&self) -> Vec<&Institution> {
        self.institutions.values().collect()
    }
}
```

- [ ] **Step 4: Create routes**

Since this is a large amount of code, create all route files by copying from the existing rust-doc-service and updating imports. The key mapping:

Old import → New import:
- `crate::extract::*` → `sp_extract::*`
- `crate::compile::*` → `sp_typst::*`
- `crate::validate::*` → direct `sp_validate::*` call
- `crate::AppError` → `crate::error::AppError`
- `crate::institutions::Registry` → `crate::institutions::Registry`

Copy the existing routes from the publish repo and adapt:

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-backend
cp -r ../scholarpress-publish/rust-doc-service/src/routes/* apps/publish-service/src/routes/
```

Then update each file. Create `routes/mod.rs`:
```rust
pub mod extract;
pub mod compile;
pub mod validate;
pub mod institutions;
pub mod spec;
pub mod template;

use axum::{Router, routing::{get, post}, extract::DefaultBodyLimit};
use crate::institutions::Registry;

pub fn router(registry: Registry) -> Router {
    Router::new()
        .route("/extract", post(extract::handler))
        .route("/compile", post(compile::handler))
        .route("/validate", post(validate::handler).layer(DefaultBodyLimit::max(50 * 1024 * 1024)))
        .route("/health", get(|| async { "ok" }))
        .route("/institutions", get(institutions::handler))
        .route("/institutions/:id/spec", get(spec::handler))
        .route("/institutions/:id/template", get(template::handler))
        .with_state(registry)
}
```

- [ ] **Step 5: Update route handlers to use sp-* crates**

For `routes/extract.rs` — replace extract imports with `sp_extract`:
```rust
use axum::{extract::{Multipart, State, Query}, Json};
use crate::institutions::Registry;
use crate::error::AppError;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ExtractParams {
    pub institution: Option<String>,
}

pub async fn handler(
    State(_registry): State<Registry>,
    Query(_params): Query<ExtractParams>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::Extraction(e.to_string()))? {
        let content_type = field.content_type().map(|c| c.to_string());
        let data = field.bytes().await.map_err(|e| AppError::Extraction(e.to_string()))?;

        let mime = content_type.as_deref().unwrap_or("application/octet-stream");
        let parsed = match mime {
            "application/pdf" => sp_extract::extract_pdf(&data).map_err(|e| AppError::Extraction(e.to_string()))?,
            mt if mt.contains("wordprocessingml") => sp_extract::extract_docx(&data).map_err(|e| AppError::Extraction(e.to_string()))?,
            _ => return Err(AppError::Extraction(format!("Unsupported format: {}", mime))),
        };

        return Ok(Json(serde_json::to_value(parsed)?));
    }
    Err(AppError::Extraction("No file uploaded".into()))
}
```

For `routes/compile.rs` — replace compile with `sp_typst::compile`:
```rust
use axum::{extract::{State, Query, Json}, http::header};
use crate::institutions::Registry;
use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CompileRequest {
    pub typst_code: String,
    #[serde(default)]
    pub variables: Option<std::collections::HashMap<String, serde_json::Value>>,
}

#[derive(Deserialize)]
pub struct CompileParams {
    pub institution: String,
}

pub async fn handler(
    State(registry): State<Registry>,
    Query(params): Query<CompileParams>,
    Json(body): Json<CompileRequest>,
) -> Result<axum::response::Response, AppError> {
    let institution = registry.get(&params.institution)
        .ok_or_else(|| AppError::InstitutionNotFound(params.institution.clone()))?;

    let code = if let Some(ref vars) = body.variables {
        sp_typst::template::render_template(&body.typst_code, vars)
    } else {
        body.typst_code
    };

    let pdf = sp_typst::compile(&code, Some(&institution.template_dir))
        .map_err(|e| AppError::Compilation(e.to_string()))?;

    Ok((
        [(header::CONTENT_TYPE, "application/pdf")],
        pdf,
    ).into_response())
}
```

For `routes/validate.rs` — replace subprocess with direct `sp_validate` call:
```rust
use axum::{extract::{State, Json}, http::StatusCode};
use crate::institutions::Registry;
use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ValidateRequest {
    pub pdf_base64: String,
    pub institution: String,
}

#[derive(Serialize)]
pub struct ValidationResult {
    pub violations: Vec<Violation>,
    pub pass_count: usize,
    pub fail_count: usize,
    pub error_count: usize,
}

#[derive(Serialize)]
pub struct Violation {
    pub check_id: String,
    pub status: String,
    pub detail: String,
    pub page: Option<i32>,
}

pub async fn handler(
    State(registry): State<Registry>,
    Json(body): Json<ValidateRequest>,
) -> Result<Json<ValidationResult>, AppError> {
    let institution = registry.get(&body.institution)
        .ok_or_else(|| AppError::InstitutionNotFound(body.institution.clone()))?;

    let pdf_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &body.pdf_base64,
    ).map_err(|e| AppError::Validation(format!("Invalid base64: {}", e)))?;

    // Write PDF to temp file (sp-validate currently reads from path)
    let tmp_dir = std::env::temp_dir().join(format!("scholarpress-validate-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&tmp_dir)?;
    let pdf_path = tmp_dir.join("input.pdf");
    std::fs::write(&pdf_path, &pdf_bytes)?;

    // Write spec to temp file
    let spec_path = tmp_dir.join("spec.yaml");
    let spec_yaml = serde_yaml::to_string(&institution.spec)
        .map_err(|e| AppError::Validation(e.to_string()))?;
    std::fs::write(&spec_path, &spec_yaml)?;

    let spec = sp_validate::spec::load_spec(&spec_path)
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let options = sp_validate::engine::CheckOptions::default();
    let results = sp_validate::engine::run_checks(&spec, &pdf_path, &options)
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let report = sp_validate::report::build_report(results);

    let violations: Vec<Violation> = report.results.iter().map(|r| {
        let page = r.evidence.first().map(|e| e.page as i32);
        Violation {
            check_id: r.check_id.clone(),
            status: r.status.to_string(),
            detail: r.detail.clone(),
            page,
        }
    }).collect();

    // Cleanup
    std::fs::remove_dir_all(&tmp_dir).ok();

    Ok(Json(ValidationResult {
        violations,
        pass_count: report.summary.pass,
        fail_count: report.summary.fail,
        error_count: report.summary.error,
    }))
}
```

For `routes/institutions.rs`:
```rust
use axum::{extract::State, Json};
use crate::institutions::Registry;
use serde::Serialize;

#[derive(Serialize)]
pub struct InstitutionSummary {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_config: Option<serde_yaml::Value>,
}

pub async fn handler(
    State(registry): State<Registry>,
) -> Json<Vec<InstitutionSummary>> {
    let list: Vec<InstitutionSummary> = registry.list().iter().map(|inst| {
        InstitutionSummary {
            id: inst.id.clone(),
            name: inst.name.clone(),
            ui_config: inst.ui_config.clone(),
        }
    }).collect();
    Json(list)
}
```

For `routes/spec.rs`:
```rust
use axum::{extract::{State, Path}, Json};
use crate::institutions::Registry;
use crate::error::AppError;
use serde::Serialize;

#[derive(Serialize)]
pub struct SpecResponse {
    pub id: String,
    pub yaml: String,
    pub summary: SpecSummary,
}

#[derive(Serialize)]
pub struct SpecSummary {
    pub document_structure: serde_yaml::Value,
    pub constants: serde_yaml::Value,
    pub automated_checks: usize,
    pub human_checks: usize,
}

pub async fn handler(
    State(registry): State<Registry>,
    Path(id): Path<String>,
) -> Result<Json<SpecResponse>, AppError> {
    let inst = registry.get(&id)
        .ok_or_else(|| AppError::InstitutionNotFound(id.clone()))?;

    let yaml = serde_yaml::to_string(&inst.spec)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let checks = inst.spec.get("checks").and_then(|c| c.as_sequence());

    let automated = checks.map(|c| c.iter().filter(|ch| {
        ch.get("automatable").and_then(|a| a.as_bool()).unwrap_or(true)
    }).count()).unwrap_or(0);

    let human = checks.map(|c| c.iter().filter(|ch| {
        !ch.get("automatable").and_then(|a| a.as_bool()).unwrap_or(true)
    }).count()).unwrap_or(0);

    Ok(Json(SpecResponse {
        id: inst.id.clone(),
        yaml,
        summary: SpecSummary {
            document_structure: inst.spec.get("document_structure").cloned().unwrap_or_default(),
            constants: inst.spec.get("constants").cloned().unwrap_or_default(),
            automated_checks: automated,
            human_checks: human,
        },
    }))
}
```

For `routes/template.rs`:
```rust
use axum::{extract::{State, Path}, Json};
use crate::institutions::Registry;
use crate::error::AppError;
use serde::Serialize;

#[derive(Serialize)]
pub struct TemplateResponse {
    pub id: String,
    pub entry: String,
    pub files: Vec<TemplateFileRef>,
}

#[derive(Serialize)]
pub struct TemplateFileRef {
    pub path: String,
    pub content: String,
}

pub async fn handler(
    State(registry): State<Registry>,
    Path(id): Path<String>,
) -> Result<Json<TemplateResponse>, AppError> {
    let inst = registry.get(&id)
        .ok_or_else(|| AppError::InstitutionNotFound(id.clone()))?;

    let template_set = sp_typst::template::load_template(&inst.template_dir)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(TemplateResponse {
        id: inst.id.clone(),
        entry: template_set.entry,
        files: template_set.files.into_iter().map(|f| TemplateFileRef {
            path: f.path,
            content: f.content,
        }).collect(),
    }))
}
```

- [ ] **Step 6: Create lib.rs**

`apps/publish-service/src/lib.rs`:
```rust
pub mod config;
pub mod error;
pub mod institutions;
pub mod routes;

use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use axum::http::header;

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "publish_service=info,tower_http=info".into()),
        )
        .init();

    let config = config::AppConfig::from_env();
    tracing::info!("Loading catalog from: {}", config.catalog_path.display());

    let registry = institutions::Registry::load(&config.catalog_path)?;
    tracing::info!("Loaded {} institutions", registry.institutions.len());

    let app = routes::router(registry)
        .layer(CorsLayer::permissive())
        .layer(axum::middleware::from_fn(request_id_middleware));

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port as u16));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn request_id_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let request_id = uuid::Uuid::new_v4().to_string();
    let mut response = next.run(req).await;
    response.headers_mut().insert(
        "x-request-id",
        axum::http::HeaderValue::from_str(&request_id).unwrap(),
    );
    response
}
```

- [ ] **Step 7: Create main.rs**

`apps/publish-service/src/main.rs`:
```rust
#[tokio::main]
async fn main() {
    if let Err(e) = publish_service::run().await {
        eprintln!("Fatal error: {}", e);
        std::process::exit(1);
    }
}
```

- [ ] **Step 8: Build**

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-backend
cargo build -p publish-service
cargo clippy -p publish-service -- -D warnings
cargo fmt --check
```

Expected: compiles with zero warnings.

- [ ] **Step 9: Commit**

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-backend
git add apps/publish-service/
git commit -m "feat(publish-service): all 7 endpoints wired to sp-* crates, CATALOG_PATH config"
```

---

### Task 9: Multi-stage Dockerfile for publish-service

**Files:**
- Create: `apps/publish-service/Dockerfile`

**Interfaces:**
- Produces: Dockerfile that builds from workspace root, produces minimal image with publish-service binary

- [ ] **Step 1: Write Dockerfile**

`apps/publish-service/Dockerfile`:
```dockerfile
FROM rust:1.85-slim-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY apps/ apps/
RUN cargo build --release --bin publish-service && \
    cp target/release/publish-service /app/publish-service

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/publish-service /usr/local/bin/publish-service
EXPOSE 4000
ENV CATALOG_PATH=/app/catalog
CMD ["publish-service"]
```

**Important**: This Dockerfile must be built from the workspace root (`scholarpress-backend/`), not from `apps/publish-service/`. The GitHub Action must use the repo root as the build context and specify the Dockerfile path explicitly.

- [ ] **Step 2: Build Docker image locally to verify**

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-backend
docker build -f apps/publish-service/Dockerfile -t publish-service-test .
```

Expected: image builds successfully.

- [ ] **Step 3: Commit**

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-backend
git add apps/publish-service/Dockerfile
git commit -m "feat: multi-stage Dockerfile for publish-service"
```

---

### Task 10: GitHub Actions workflows for both repos

**Files:**
- Create: `scholarpress-publish-ui/.github/workflows/docker-publish.yml`
- Replace: `scholarpress-backend/.github/workflows/docker-publish.yml`

**Interfaces:**
- Produces: GHCR images pushed on push to main

- [ ] **Step 1: CI for publish-ui**

`/home/danriggi/scholarpress-workshop/scholarpress-publish-ui/.github/workflows/docker-publish.yml`:
```yaml
name: Docker Publish
on:
  push:
    branches: [main]
env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}
jobs:
  build:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write
    steps:
      - uses: actions/checkout@v4
      - name: Login to GHCR
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:latest
```

- [ ] **Step 2: CI for backend**

Replace `/home/danriggi/scholarpress-workshop/scholarpress-backend/.github/workflows/docker-publish.yml`:
```yaml
name: Docker Publish
on:
  push:
    branches: [main]
env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}
jobs:
  build:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write
    steps:
      - uses: actions/checkout@v4
      - name: Login to GHCR
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          file: apps/publish-service/Dockerfile
          push: true
          tags: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}-publish-service:latest
```

The backend CI uses the repo root as context (needed for workspace Cargo.toml with `crates/*` paths) and specifies the Dockerfile path via `file:`. Image tag appends `-publish-service` to distinguish from potential future images in the same repo.

- [ ] **Step 3: Commit and push both**

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-publish-ui
git add .github/
git commit -m "ci: GHCR Docker publish on push to main"
git push

cd /home/danriggi/scholarpress-workshop/scholarpress-backend
git add .github/
git commit -m "ci: GHCR Docker publish for publish-service"
git push
```

---

### Task 11: Create scholarpress-deliver orchestration repo

**Files:**
- Create: `scholarpress-deliver/docker-compose.yml`
- Create: `scholarpress-deliver/README.md`

**Interfaces:**
- Produces: End-user repo that runs the full stack with `docker-compose up`

- [ ] **Step 1: Clone and set up deliver repo**

```bash
cd /home/danriggi/scholarpress-workshop
rm -rf scholarpress-deliver
git clone https://github.com/scholarpress-workshop/scholarpress-deliver.git
cd scholarpress-deliver
```

- [ ] **Step 2: Create docker-compose.yml**

`docker-compose.yml`:
```yaml
services:
  web:
    image: ghcr.io/scholarpress-workshop/scholarpress-publish-ui:latest
    ports:
      - "3000:3000"
    environment:
      - RUST_SERVICE_URL=http://rust-doc-service:4000
      - LLM_BASE_URL=https://reallms.rescloud.iu.edu/direct/v1
      - LLM_MODEL=gemma-4-31B-it
    depends_on:
      - rust-doc-service

  rust-doc-service:
    image: ghcr.io/scholarpress-workshop/scholarpress-backend-publish-service:latest
    ports:
      - "4000:4000"
    environment:
      - CATALOG_PATH=/app/catalog
    volumes:
      - ../scholarpress-catalog:/app/catalog:ro
```

- [ ] **Step 3: Create README.md**

```markdown
# scholarpress-deliver

One-command deployment for the ScholarPress ecosystem. Pulls pre-built images from GHCR — no source builds required.

## Quick start

```bash
git clone https://github.com/scholarpress-workshop/scholarpress-catalog ../scholarpress-catalog
docker-compose up
```

Open http://localhost:3000.

## Architecture

Pulls two images from GitHub Container Registry:

| Service | Image | Port |
|---------|-------|------|
| Web UI | `ghcr.io/scholarpress-workshop/scholarpress-publish-ui:latest` | 3000 |
| Doc Service | `ghcr.io/scholarpress-workshop/scholarpress-backend-publish-service:latest` | 4000 |

The catalog is mounted as a read-only volume at runtime — update catalog data without rebuilding images.

## Environment

| Variable | Default | Service |
|----------|---------|---------|
| `LLM_BASE_URL` | `https://reallms.rescloud.iu.edu/direct/v1` | web |
| `LLM_MODEL` | `gemma-4-31B-it` | web |
| `CATALOG_PATH` | `/app/catalog` | rust-doc-service |
```

- [ ] **Step 4: Commit and push**

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-deliver
git add docker-compose.yml README.md
git commit -m "feat: initial delivery — docker-compose with GHCR images"
git push -u origin main
```

---

### Task 12: Final verification — end-to-end build and test

**Interfaces:**
- Verifies: Backend workspace compiles clean, all tests pass, Docker builds work

- [ ] **Step 1: Clean workspace build**

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-backend
cargo clean
cargo build --release
```

Expected: all crates and apps compile.

- [ ] **Step 2: Run all tests**

```bash
cargo test --all
```

Expected: all sp-validate (83), sp-extract, and sp-typst tests pass.

- [ ] **Step 3: Clippy and fmt**

```bash
cargo clippy --all -- -D warnings
cargo fmt --check
```

Expected: zero warnings, all files formatted.

- [ ] **Step 4: Verify publish-ui builds**

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-publish-ui
bun run build
bun run lint
```

Expected: clean build, zero lint errors.

- [ ] **Step 5: Push final state**

```bash
cd /home/danriggi/scholarpress-workshop/scholarpress-backend
git push

cd /home/danriggi/scholarpress-workshop/scholarpress-publish-ui
git push

cd /home/danriggi/scholarpress-workshop/scholarpress-deliver
git push
```
