#[derive(Debug, Clone)]
pub struct TextSpan {
    pub text: String,
    pub font_name: String,
    pub font_size: f32,
    pub bbox: (f32, f32, f32, f32),
    pub is_bold: bool,
    pub is_italic: bool,
}

#[derive(Debug, Clone)]
pub struct Page {
    pub page_number: usize,
    pub width: f32,
    pub height: f32,
    pub spans: Vec<TextSpan>,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub pages: Vec<Page>,
}
