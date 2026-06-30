## Helper functions available across codebase

### In checkers/content.py
- `_page_lines(page)` → list of line strings grouped by top position
- `_page_text(page)` → full page text as single lowercase string
- `_page_text_no_citations(page)` → page text excluding DOI/URL lines and numbered reference lines
- `_extract_toc_entries(page)` → list of (title, page_number) tuples from TOC
- `_extract_page_heading(page)` → heading text from page (chapter/appendix detection)
- `_normalize_title(title)` → lowercase, strip punctuation, normalize dashes

### In checkers/typography.py
- `_parse_measurement("1in")` → 72.0, `_parse_measurement("12pt")` → 12.0
- `_detect_weight(font_name)` → "normal", "bold", "italic", "bold-italic"
- `_normalize_family(font_name)` → "TimesNewRoman", "Arial" (strips prefix, PS/MT suffixes, style variants)
- `_check_bold(font_name)`, `_check_italic(font_name)` → bool

### In checkers/layout.py
- `_parse_measurement(value)` → float in points

### TOC parsing usage
```python
entries = _extract_toc_entries(toc_page)
# Returns: [("Chapter 1: Introduction", 5), ...]
```
