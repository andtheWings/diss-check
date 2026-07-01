# diss-check

<img src="hex.png" alt="diss-check hex sticker" width="180" align="right">

Automated dissertation formatting compliance checker. PDF in, violations out.

Extracts text, fonts, and layout from PDFs using [pdf_oxide](https://github.com/yfedoseev/pdf_oxide), then runs automated checkers against institution-specific formatting requirements defined in YAML.

## Quick Start

```bash
cargo install diss-check
diss-check check --spec specs/iu.yaml dissertation.pdf
```

## Installation

### From crates.io
```bash
cargo install diss-check
```

### From source
```bash
git clone https://github.com/danriggi/diss-check
cd diss-check
cargo build --release
```

## Usage

### Check a single PDF
```bash
diss-check check --spec specs/iu.yaml dissertation.pdf
```

### JSON output
```bash
diss-check check --spec specs/iu.yaml dissertation.pdf --json
```

### Run a specific check
```bash
diss-check check --spec specs/iu.yaml --check global_margins dissertation.pdf
```

### Run all checks in a category
```bash
diss-check check --spec specs/iu.yaml --category typography dissertation.pdf
```

### Show only failures
```bash
diss-check check --spec specs/iu.yaml --quiet dissertation.pdf
```

### Calibration (corpus analysis)
```bash
diss-check calibrate --spec specs/iu.yaml --corpus tests/corpus/
```

### Exit codes
- `0` — all checks PASS
- `1` — one or more FAIL or ERROR
- `2` — usage error (missing file, invalid spec)

## Spec Format

Institution formatting requirements are defined in YAML:

```yaml
institution: Indiana University
source_revision: September 2025

checks:
  - id: global_margins
    category: layout
    checker: margins
    target:
      scope: all_pages
    params:
      top: 1in
      bottom: 1in
      left: 1.25in
      right: 1.25in

  - id: font_size_consistent
    category: typography
    checker: font_size
    target:
      scope: all_pages
    params:
      allowed: ["10pt", "11pt", "12pt"]
      consistent: true

  - id: some_human_check
    category: human
    checker: review
    target:
      page: title_page
    params:
      prompt: Verify title is centered and in all caps
```

### Checker catalog

| Checker | Category | What it checks |
|---------|----------|---------------|
| `margins` | layout | Statistical margin-setting compliance (percentile-based) |
| `margin_symmetry` | layout | Per-page left/right margin symmetry |
| `font_size` | typography | Font size within allowed range, body consistency |
| `font_weight` | typography | Bold/italic usage (with page filter + invert) |
| `font_family` | typography | Font family consistency (semantic exclusions for math/code) |
| `justification` | typography | Left/justified consistency (front matter excluded) |
| `section_presence` | structure | Required sections present |
| `section_order` | structure | Sections in correct order |
| `page_numbers_format` | structure | Roman in front matter, Arabic in body |
| `headings_consistent` | structure | Headings use same font as body |
| `new_chapters_new_pages` | structure | Chapters start on new pages |
| `boilerplate_match` | content | Template text with {variable} substitution (70% threshold) |
| `committee_order` | content | Chair listed first on acceptance page |
| `toc_title_parity` | content | TOC entries match body chapter headings |
| `review` | human | Manual review with configurable prompt |

Full checker list: 33 automated + 7 human review. See `specs/iu.yaml` for all check definitions.

## Architecture

```
diss-check/
  src/
    main.rs        # CLI (clap)
    engine.rs      # Check runner
    spec.rs         # YAML spec loader
    extractor.rs    # pdf_oxide PDF extraction
    document.rs     # TextSpan/Page/Document model
    report.rs       # Text + JSON report formatting
    calibration.rs  # Corpus calibration workflow
    checkers/
      layout.rs     # margins, margin_symmetry
      typography.rs  # font_size, font_weight, font_family, justification
      structure.rs   # section_presence, section_order, page numbers, headings
      content.rs     # boilerplate, committee, toc_title_parity
      title_page.rs  # all_caps, clause_centered, clause_spacing
      sections.rs    # references_font, cv, abstract checkers
      footnotes.rs   # footnotes_font
      optional_pages.rs # copyright_page
      toc_details.rs # toc_page_numbers, toc_overhang, toc_cv_no_dots
```

## Development

```bash
cargo build --release    # 0 warnings required
cargo test               # 83 tests
cargo clippy             # Lint
cargo fmt --check        # Format check
```

## License

MIT
