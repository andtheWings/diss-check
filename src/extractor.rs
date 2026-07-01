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

        let chars = doc.extract_chars(page_index)?;

        let mut spans: Vec<TextSpan> = Vec::new();
        let mut word_chars: Vec<&pdf_oxide::layout::TextChar> = Vec::new();

        for ch in &chars {
            let is_space = ch.char.is_whitespace();

            if is_space {
                if !word_chars.is_empty() {
                    spans.push(build_word(&word_chars, height));
                    word_chars.clear();
                }
            } else {
                if !word_chars.is_empty() {
                    let last = word_chars.last().unwrap();
                    let same_line = (last.origin_y - ch.origin_y).abs() < 3.0;
                    let gap = ch.bbox.x - (last.bbox.x + last.bbox.width);
                    let same_font = last.font_name == ch.font_name
                        && (last.font_size - ch.font_size).abs() < 1.0;

                    if !same_line || !same_font || gap > 20.0 {
                        spans.push(build_word(&word_chars, height));
                        word_chars.clear();
                    }
                }
                word_chars.push(ch);
            }
        }
        if !word_chars.is_empty() {
            spans.push(build_word(&word_chars, height));
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

fn build_word(chars: &[&pdf_oxide::layout::TextChar], page_height: f32) -> TextSpan {
    let first = chars[0];
    let text: String = chars.iter().map(|c| c.char).collect();

    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;

    for ch in chars {
        let x = ch.bbox.x;
        let y = ch.bbox.y;
        let w = ch.bbox.width;
        let h = ch.bbox.height;
        min_x = min_x.min(x);
        max_x = max_x.max(x + w);
        min_y = min_y.min(y);
        max_y = max_y.max(y + h);
    }

    let top = page_height - max_y;
    let bottom = page_height - min_y;

    TextSpan {
        text,
        font_name: first.font_name.clone(),
        font_size: first.font_size,
        bbox: (top.max(0.0), bottom, min_x, max_x),
        is_bold: matches!(first.font_weight, pdf_oxide::layout::FontWeight::Bold),
        is_italic: first.is_italic,
    }
}
