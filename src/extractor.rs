use std::path::Path;
use crate::document::{Document, Page, TextSpan};

pub fn extract_document(path: &Path) -> Result<Document, Box<dyn std::error::Error>> {
    let doc = pdf_oxide::PdfDocument::open(path)?;
    let page_count = doc.page_count()?;
    let mut pages: Vec<Page> = Vec::with_capacity(page_count);

    for page_index in 0..page_count {
        let (llx, _lly, urx, ury) = doc.get_page_media_box(page_index)?;
        let width = urx - llx;
        let height = ury - _lly;

        let pdf_spans = doc.extract_spans(page_index)?;

        let mut spans: Vec<TextSpan> = Vec::new();
        for ps in &pdf_spans {
            let is_bold = matches!(ps.font_weight, pdf_oxide::layout::FontWeight::Bold);
            let top = height - (ps.bbox.y + ps.bbox.height);
            let bottom = height - ps.bbox.y;

            let words: Vec<&str> = ps.text.split_whitespace().collect();

            if words.is_empty() {
                spans.push(TextSpan {
                    text: ps.text.clone(),
                    font_name: ps.font_name.clone(),
                    font_size: ps.font_size,
                    bbox: (top.min(bottom).max(0.0), top.max(bottom), ps.bbox.x, ps.bbox.x + ps.bbox.width),
                    is_bold,
                    is_italic: ps.is_italic,
                });
                continue;
            }

            let full_text: String = words.join(" ");
            let mut byte_offset = 0usize;

            for word in &words {
                let start = ps.text[byte_offset..].find(word).unwrap_or(0);
                let word_start = byte_offset + start;

                let char_before: usize = ps.text[..word_start].chars().count();
                let char_in_word: usize = word.chars().count();
                let total_chars: usize = ps.text.chars().count().max(1);

                let frac_start = char_before as f32 / total_chars as f32;
                let frac_end = (char_before + char_in_word) as f32 / total_chars as f32;

                let word_x0 = ps.bbox.x + (ps.bbox.width * frac_start);
                let word_x1 = ps.bbox.x + (ps.bbox.width * frac_end);

                spans.push(TextSpan {
                    text: word.to_string(),
                    font_name: ps.font_name.clone(),
                    font_size: ps.font_size,
                    bbox: (top, bottom, word_x0, word_x1),
                    is_bold,
                    is_italic: ps.is_italic,
                });

                byte_offset = word_start + word.len();
            }
        }

        pages.push(Page {
            page_number: page_index + 1,
            width,
            height,
            spans,
        });
    }

    Ok(Document { pages })
}
