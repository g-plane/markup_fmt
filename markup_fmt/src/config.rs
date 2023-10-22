//! Types about configuration.

#[cfg(feature = "config_serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(rename_all = "camelCase", default))]
/// The whole configuration of markup_fmt.
///
/// For detail, please refer to [Configuration](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md) on GitHub.
pub struct FormatOptions {
    #[cfg_attr(feature = "config_serde", serde(flatten))]
    pub layout: LayoutOptions,
    #[cfg_attr(feature = "config_serde", serde(flatten))]
    pub language: LanguageOptions,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(rename_all = "camelCase", default))]
/// Configuration related to layout, such as indentation or print width.
pub struct LayoutOptions {
    /// See [`printWidth`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#printwidth) on GitHub
    pub print_width: usize,
    /// See [`useTabs`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#usetabs) on GitHub
    pub use_tabs: bool,
    /// See [`indentWidth`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#indentwidth) on GitHub
    pub indent_width: usize,
    /// See [`lineBreak`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#linebreak) on GitHub
    pub line_break: LineBreak,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            print_width: 80,
            use_tabs: false,
            indent_width: 2,
            line_break: LineBreak::Lf,
        }
    }
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(rename_all = "camelCase"))]
pub enum LineBreak {
    #[default]
    Lf,
    Crlf,
}

impl From<LineBreak> for tiny_pretty::LineBreak {
    fn from(value: LineBreak) -> Self {
        match value {
            LineBreak::Lf => tiny_pretty::LineBreak::Lf,
            LineBreak::Crlf => tiny_pretty::LineBreak::Crlf,
        }
    }
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(rename_all = "camelCase", default))]
/// Configuration related to syntax.
pub struct LanguageOptions {
    pub quotes: Quotes,
    pub format_comments: bool,
    pub script_indent: bool,
    #[cfg_attr(feature = "config_serde", serde(rename = "html.scriptIndent"))]
    pub html_script_indent: Option<bool>,
    #[cfg_attr(feature = "config_serde", serde(rename = "vue.scriptIndent"))]
    pub vue_script_indent: Option<bool>,
    #[cfg_attr(feature = "config_serde", serde(rename = "svelte.scriptIndent"))]
    pub svelte_script_indent: Option<bool>,
    pub style_indent: bool,
    #[cfg_attr(feature = "config_serde", serde(rename = "html.styleIndent"))]
    pub html_style_indent: Option<bool>,
    #[cfg_attr(feature = "config_serde", serde(rename = "vue.styleIndent"))]
    pub vue_style_indent: Option<bool>,
    #[cfg_attr(feature = "config_serde", serde(rename = "svelte.styleIndent"))]
    pub svelte_style_indent: Option<bool>,
    pub closing_bracket_same_line: bool,
    pub closing_tag_line_break_for_empty: ClosingTagLineBreakForEmpty,
    pub v_bind_style: Option<VBindStyle>,
    pub v_on_style: Option<VOnStyle>,
    pub v_for_delimiter_style: Option<VForDelimiterStyle>,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(rename_all = "camelCase"))]
pub enum Quotes {
    #[default]
    Double,
    Single,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(rename_all = "camelCase"))]
pub enum ClosingTagLineBreakForEmpty {
    Always,
    #[default]
    Fit,
    Never,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(rename_all = "camelCase"))]
pub enum VBindStyle {
    #[default]
    Short,
    Long,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(rename_all = "camelCase"))]
pub enum VOnStyle {
    #[default]
    Short,
    Long,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(rename_all = "camelCase"))]
pub enum VForDelimiterStyle {
    #[default]
    In,
    Of,
}
