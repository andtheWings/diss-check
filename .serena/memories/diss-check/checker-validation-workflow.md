## Checker Development & Validation Workflow

For each checker round:
1. **Design** based on `tests/fixtures/2020-12-chambers.pdf`
2. **Implement** the checker
3. **Validate** against `tests/fixtures/2025-06-alexander.pdf` with human-in-the-loop feedback
4. **Reference** these IU artifacts for violation assessments:
   - `specs/artifacts/iu/format-review-checklist.pdf` (extracted via pdfplumber)
   - `specs/artifacts/iu/formatting-template.docx` (extracted via python-docx or zipfile/xml)

Key IU requirements from artifacts:
- Font size: 11pt or 12pt, pick one, consistent throughout
- Title page: "Same font size as document" (NOT larger)
- Headings: "same font type and size as the rest of the document"
- Footnotes: ≥ 10pt, ≤ document font size
- Figure/table legends/descriptors: same as document or ≥ 10pt
- Figure/table content: may be smaller if legible (known limitation in font_size checker)
- Table of Contents: "Each level's format will match"
- References: same font size/type as document
- Page numbers: same font/size as document, 0.5" from bottom, centered
