## Architecture

```
diss_check/
  cli.py           # click CLI: --spec + PDF argument, --json flag
  spec.py           # InstitutionSpec pydantic model, load_spec() from YAML
  engine.py         # Engine: collects extractors, instantiates checkers, runs checks
  document.py       # TextSpan, Page, Document, ExtractionContext models
  report.py         # Report, Summary, format_text(), format_json()
  extractors/
    pdfplumber_extractor.py  # Groups chars into word-level TextSpans with font data
  checkers/
    base.py          # BaseChecker ABC, CheckResult, EvidenceItem, register_checker()
    layout.py        # MarginsChecker
    typography.py    # FontSizeChecker, FontWeightChecker, FontFamilyChecker, JustificationChecker
    structure.py     # SectionPresenceChecker, SectionOrderChecker
    content.py       # BoilerplateMatchChecker, CommitteeOrderChecker, TocTitleParityChecker
    human.py         # HumanReviewChecker (always returns MANUAL)
```

## Key patterns

### Checker registration
```python
@register_checker(category="typography", name="font_size")
class FontSizeChecker(BaseChecker):
    requires = ["pdfplumber"]
    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult: ...
```

### Document model
- `TextSpan(text, font_name, font_size, bbox=(top, bottom, x0, x1))`
- `TextSpan.top/bbox/x0/x1` properties return bbox[0]/[1]/[2]/[3] respectively
- `Page(page_number, width, height, spans)`
- `Document(pages)`
- `ExtractionContext(document, docling_doc)`

### Test pattern
```python
def _make_doc(spans_by_page):
    # Each element = [(bbox, text), ...] or [(bbox, text, font_size), ...]
    # One inner list per page
    pages = []
    for page_spans in spans_by_page:
        spans = [TextSpan(text=t, font_name="Times", font_size=fs, bbox=bbox) for bbox, t, fs in page_spans]
        pages.append(Page(page_number=len(pages)+1, width=612, height=792, spans=spans))
    return Document(pages=pages)
```

### Common zone filters
- Header zone: `span.top < 36` (0.5in)
- Page number zone: `span.bottom > (page.height - 50)` (0.7in from bottom)
- Skip whitespace: `if not span.text.strip(): continue`
- Skip small font artifacts: `if font_size < 8.5: continue`

### Engine flow
1. Load spec YAML → InstitutionSpec
2. Collect required extractors from checker.requires
3. Run extractors → populate ExtractionContext
4. For each check_def: get checker instance, call checker.check(ctx, params)
5. Non-automatable checks (check_def.automatable == False) return MANUAL directly

## Validation workflow
See mem:diss-check/checker-validation-workflow for the full process including artifact references.
