//! Types about configuration.

#[cfg(feature = "config_serde")]
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;

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
    /// See [`quotes`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#quotes) on GitHub
    pub quotes: Quotes,

    /// See [`formatComments`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#formatcomments) on GitHub
    pub format_comments: bool,

    /// See [`scriptIndent`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#scriptindent) on GitHub
    pub script_indent: bool,
    #[cfg_attr(feature = "config_serde", serde(rename = "html.scriptIndent"))]
    /// See [`scriptIndent`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#scriptindent) on GitHub
    pub html_script_indent: Option<bool>,
    #[cfg_attr(feature = "config_serde", serde(rename = "vue.scriptIndent"))]
    /// See [`scriptIndent`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#scriptindent) on GitHub
    pub vue_script_indent: Option<bool>,
    #[cfg_attr(feature = "config_serde", serde(rename = "svelte.scriptIndent"))]
    /// See [`scriptIndent`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#scriptindent) on GitHub
    pub svelte_script_indent: Option<bool>,

    /// See [`styleIndent`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#styleindent) on GitHub
    pub style_indent: bool,
    #[cfg_attr(feature = "config_serde", serde(rename = "html.styleIndent"))]
    /// See [`styleIndent`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#styleindent) on GitHub
    pub html_style_indent: Option<bool>,
    #[cfg_attr(feature = "config_serde", serde(rename = "vue.styleIndent"))]
    /// See [`styleIndent`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#styleindent) on GitHub
    pub vue_style_indent: Option<bool>,
    #[cfg_attr(feature = "config_serde", serde(rename = "svelte.styleIndent"))]
    /// See [`styleIndent`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#styleindent) on GitHub
    pub svelte_style_indent: Option<bool>,

    /// See [`closingBracketSameLine`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#closingbracketsameline) on GitHub
    pub closing_bracket_same_line: bool,

    /// See [`closingTagLineBreakForEmpty`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#closingtaglinebreakforempty) on GitHub
    pub closing_tag_line_break_for_empty: ClosingTagLineBreakForEmpty,

    /// See [`maxAttrsPerLine`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#maxattrsperline) on GitHub
    pub max_attrs_per_line: Option<NonZeroUsize>,

    #[cfg_attr(feature = "config_serde", serde(rename = "html.normal.selfClosing"))]
    /// See [`*.selfClosing`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#selfclosing) on GitHub
    pub html_normal_self_closing: Option<bool>,
    #[cfg_attr(feature = "config_serde", serde(rename = "html.void.selfClosing"))]
    /// See [`*.selfClosing`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#selfclosing) on GitHub
    pub html_void_self_closing: Option<bool>,
    #[cfg_attr(feature = "config_serde", serde(rename = "component.selfClosing"))]
    /// See [`*.selfClosing`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#selfclosing) on GitHub
    pub component_self_closing: Option<bool>,
    #[cfg_attr(feature = "config_serde", serde(rename = "svg.selfClosing"))]
    /// See [`*.selfClosing`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#selfclosing) on GitHub
    pub svg_self_closing: Option<bool>,
    #[cfg_attr(feature = "config_serde", serde(rename = "mathml.selfClosing"))]
    /// See [`*.selfClosing`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#selfclosing) on GitHub
    pub mathml_self_closing: Option<bool>,

    /// See [`whitespaceSensitivity`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#whitespacesensitivity) on GitHub
    pub whitespace_sensitivity: WhitespaceSensitivity,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "component.whitespaceSensitivity")
    )]
    /// See [`whitespaceSensitivity`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#whitespacesensitivity) on GitHub
    pub component_whitespace_sensitivity: Option<WhitespaceSensitivity>,

    /// See [`vBindStyle`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#vbindstyle) on GitHub
    pub v_bind_style: Option<VBindStyle>,
    /// See [`vOnStyle`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#vonstyle) on GitHub
    pub v_on_style: Option<VOnStyle>,
    /// See [`vForDelimiterStyle`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#vfordelimiterstyle) on GitHub
    pub v_for_delimiter_style: Option<VForDelimiterStyle>,
    /// See [`vSlotStyle`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#vslotstyle) on GitHub
    pub v_slot_style: Option<VSlotStyle>,
    #[cfg_attr(feature = "config_serde", serde(rename = "component.vSlotStyle"))]
    /// See [`vSlotStyle`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#vslotstyle) on GitHub
    pub component_v_slot_style: Option<VSlotStyle>,
    #[cfg_attr(feature = "config_serde", serde(rename = "default.vSlotStyle"))]
    /// See [`vSlotStyle`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#vslotstyle) on GitHub
    pub default_v_slot_style: Option<VSlotStyle>,
    #[cfg_attr(feature = "config_serde", serde(rename = "named.vSlotStyle"))]
    /// See [`vSlotStyle`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#vslotstyle) on GitHub
    pub named_v_slot_style: Option<VSlotStyle>,
    /// See [`vBindSameNameShortHand`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#vbindsamenameshorthand) on GitHub
    pub v_bind_same_name_short_hand: Option<bool>,

    /// See [`strictSvelteAttr`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#strictsvelteattr) on GitHub
    pub strict_svelte_attr: bool,
    /// See [`svelteAttrShorthand`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#svelteattrshorthand) on GitHub
    pub svelte_attr_shorthand: Option<bool>,
    /// See [`svelteDirectiveShorthand`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#sveltedirectiveshorthand) on GitHub
    pub svelte_directive_shorthand: Option<bool>,
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
pub enum WhitespaceSensitivity {
    #[default]
    Css,
    Strict,
    Ignore,
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

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(rename_all = "camelCase"))]
pub enum VSlotStyle {
    #[default]
    Short,
    Long,
    VSlot,
}
