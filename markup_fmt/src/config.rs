#[derive(Clone, Default)]
pub struct LanguageOptions {
    pub script_indent: bool,
    pub style_indent: bool,
    pub v_bind_style: Option<VBindStyle>,
    pub v_on_style: Option<VOnStyle>,
}

#[derive(Clone, Default)]
pub enum VBindStyle {
    #[default]
    Short,
    Long,
}

#[derive(Clone, Default)]
pub enum VOnStyle {
    #[default]
    Short,
    Long,
}
