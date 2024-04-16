//! Types about configuration.

#[cfg(feature = "config_serde")]
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(default))]
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
#[cfg_attr(feature = "config_serde", serde(default))]
/// Configuration related to layout, such as indentation or print width.
pub struct LayoutOptions {
    #[cfg_attr(feature = "config_serde", serde(alias = "printWidth"))]
    /// See [`printWidth`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#printwidth) on GitHub
    pub print_width: usize,

    #[cfg_attr(feature = "config_serde", serde(alias = "useTabs"))]
    /// See [`useTabs`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#usetabs) on GitHub
    pub use_tabs: bool,

    #[cfg_attr(feature = "config_serde", serde(alias = "indentWidth"))]
    /// See [`indentWidth`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#indentwidth) on GitHub
    pub indent_width: usize,

    #[cfg_attr(
        feature = "config_serde",
        serde(alias = "lineBreak", alias = "linebreak")
    )]
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
#[cfg_attr(feature = "config_serde", serde(rename_all = "kebab-case"))]
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

#[derive(Clone, Debug)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(default))]
/// Configuration related to syntax.
pub struct LanguageOptions {
    /// See [`quotes`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#quotes) on GitHub
    pub quotes: Quotes,

    #[cfg_attr(feature = "config_serde", serde(alias = "formatComments"))]
    /// See [`formatComments`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#formatcomments) on GitHub
    pub format_comments: bool,

    #[cfg_attr(feature = "config_serde", serde(alias = "scriptIndent"))]
    /// See [`scriptIndent`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#scriptindent) on GitHub
    pub script_indent: bool,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "html.script_indent", alias = "html.scriptIndent")
    )]
    /// See [`scriptIndent`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#scriptindent) on GitHub
    pub html_script_indent: Option<bool>,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "vue.script_indent", alias = "vue.scriptIndent")
    )]
    /// See [`scriptIndent`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#scriptindent) on GitHub
    pub vue_script_indent: Option<bool>,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "svelte.script_indent", alias = "svelte.scriptIndent")
    )]
    /// See [`scriptIndent`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#scriptindent) on GitHub
    pub svelte_script_indent: Option<bool>,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "astro.script_indent", alias = "astro.scriptIndent")
    )]
    /// See [`scriptIndent`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#scriptindent) on GitHub
    pub astro_script_indent: Option<bool>,

    #[cfg_attr(feature = "config_serde", serde(alias = "styleIndent"))]
    /// See [`styleIndent`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#styleindent) on GitHub
    pub style_indent: bool,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "html.style_indent", alias = "html.styleIndent")
    )]
    /// See [`styleIndent`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#styleindent) on GitHub
    pub html_style_indent: Option<bool>,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "vue.style_indent", alias = "vue.styleIndent")
    )]
    /// See [`styleIndent`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#styleindent) on GitHub
    pub vue_style_indent: Option<bool>,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "svelte.style_indent", alias = "svelte.styleIndent")
    )]
    /// See [`styleIndent`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#styleindent) on GitHub
    pub svelte_style_indent: Option<bool>,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "astro.style_indent", alias = "astro.styleIndent")
    )]
    /// See [`styleIndent`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#styleindent) on GitHub
    pub astro_style_indent: Option<bool>,

    #[cfg_attr(feature = "config_serde", serde(alias = "closingBracketSameLine"))]
    /// See [`closingBracketSameLine`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#closingbracketsameline) on GitHub
    pub closing_bracket_same_line: bool,

    #[cfg_attr(feature = "config_serde", serde(alias = "closingTagLineBreakForEmpty"))]
    /// See [`closingTagLineBreakForEmpty`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#closingtaglinebreakforempty) on GitHub
    pub closing_tag_line_break_for_empty: ClosingTagLineBreakForEmpty,

    #[cfg_attr(feature = "config_serde", serde(alias = "maxAttrsPerLine"))]
    /// See [`maxAttrsPerLine`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#maxattrsperline) on GitHub
    pub max_attrs_per_line: Option<NonZeroUsize>,

    #[cfg_attr(feature = "config_serde", serde(alias = "preferAttrsSingleLine"))]
    /// See [`preferAttrsSingleLine`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#preferattrssingleline) on GitHub
    pub prefer_attrs_single_line: bool,

    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "html.normal.self_closing", alias = "html.normal.selfClosing")
    )]
    /// See [`*.selfClosing`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#selfclosing) on GitHub
    pub html_normal_self_closing: Option<bool>,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "html.void.self_closing", alias = "html.void.selfClosing")
    )]
    /// See [`*.selfClosing`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#selfclosing) on GitHub
    pub html_void_self_closing: Option<bool>,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "component.self_closing", alias = "component.selfClosing")
    )]
    /// See [`*.selfClosing`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#selfclosing) on GitHub
    pub component_self_closing: Option<bool>,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "svg.self_closing", alias = "svg.selfClosing")
    )]
    /// See [`*.selfClosing`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#selfclosing) on GitHub
    pub svg_self_closing: Option<bool>,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "mathml.self_closing", alias = "mathml.selfClosing")
    )]
    /// See [`*.selfClosing`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#selfclosing) on GitHub
    pub mathml_self_closing: Option<bool>,

    #[cfg_attr(feature = "config_serde", serde(alias = "whitespaceSensitivity"))]
    /// See [`whitespaceSensitivity`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#whitespacesensitivity) on GitHub
    pub whitespace_sensitivity: WhitespaceSensitivity,
    #[cfg_attr(
        feature = "config_serde",
        serde(
            rename = "component.whitespace_sensitivity",
            alias = "component.whitespaceSensitivity"
        )
    )]
    /// See [`whitespaceSensitivity`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#whitespacesensitivity) on GitHub
    pub component_whitespace_sensitivity: Option<WhitespaceSensitivity>,

    #[cfg_attr(feature = "config_serde", serde(alias = "vBindStyle"))]
    /// See [`vBindStyle`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#vbindstyle) on GitHub
    pub v_bind_style: Option<VBindStyle>,
    #[cfg_attr(feature = "config_serde", serde(alias = "vOnStyle"))]
    /// See [`vOnStyle`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#vonstyle) on GitHub
    pub v_on_style: Option<VOnStyle>,
    #[cfg_attr(feature = "config_serde", serde(alias = "vForDelimiterStyle"))]
    /// See [`vForDelimiterStyle`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#vfordelimiterstyle) on GitHub
    pub v_for_delimiter_style: Option<VForDelimiterStyle>,
    #[cfg_attr(feature = "config_serde", serde(alias = "vSlotStyle"))]
    /// See [`vSlotStyle`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#vslotstyle) on GitHub
    pub v_slot_style: Option<VSlotStyle>,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "component.v_slot_style", alias = "component.vSlotStyle")
    )]
    /// See [`vSlotStyle`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#vslotstyle) on GitHub
    pub component_v_slot_style: Option<VSlotStyle>,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "default.v_slot_style", alias = "default.vSlotStyle")
    )]
    /// See [`vSlotStyle`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#vslotstyle) on GitHub
    pub default_v_slot_style: Option<VSlotStyle>,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "named.v_slot_style", alias = "named.vSlotStyle")
    )]
    /// See [`vSlotStyle`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#vslotstyle) on GitHub
    pub named_v_slot_style: Option<VSlotStyle>,
    #[cfg_attr(feature = "config_serde", serde(alias = "vBindSameNameShortHand"))]
    /// See [`vBindSameNameShortHand`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#vbindsamenameshorthand) on GitHub
    pub v_bind_same_name_short_hand: Option<bool>,

    #[cfg_attr(feature = "config_serde", serde(alias = "strictSvelteAttr"))]
    /// See [`strictSvelteAttr`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#strictsvelteattr) on GitHub
    pub strict_svelte_attr: bool,
    #[cfg_attr(feature = "config_serde", serde(alias = "svelteAttrShorthand"))]
    /// See [`svelteAttrShorthand`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#svelteattrshorthand) on GitHub
    pub svelte_attr_shorthand: Option<bool>,
    #[cfg_attr(feature = "config_serde", serde(alias = "svelteDirectiveShorthand"))]
    /// See [`svelteDirectiveShorthand`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#sveltedirectiveshorthand) on GitHub
    pub svelte_directive_shorthand: Option<bool>,

    #[cfg_attr(feature = "config_serde", serde(alias = "astroAttrShorthand"))]
    /// See [`astroAttrShorthand`](https://github.com/g-plane/markup_fmt/blob/main/docs/config.md#astroattrshorthand) on GitHub
    pub astro_attr_shorthand: Option<bool>,
}

impl Default for LanguageOptions {
    fn default() -> Self {
        Self {
            quotes: Quotes::default(),
            format_comments: false,
            script_indent: true,
            html_script_indent: None,
            vue_script_indent: Some(false),
            svelte_script_indent: None,
            astro_script_indent: None,
            style_indent: true,
            html_style_indent: None,
            vue_style_indent: Some(false),
            svelte_style_indent: None,
            astro_style_indent: None,
            closing_bracket_same_line: false,
            closing_tag_line_break_for_empty: ClosingTagLineBreakForEmpty::default(),
            max_attrs_per_line: None,
            prefer_attrs_single_line: false,
            html_normal_self_closing: None,
            html_void_self_closing: Some(false),
            component_self_closing: None,
            svg_self_closing: None,
            mathml_self_closing: None,
            whitespace_sensitivity: WhitespaceSensitivity::default(),
            component_whitespace_sensitivity: None,
            v_bind_style: None,
            v_on_style: None,
            v_for_delimiter_style: None,
            v_slot_style: None,
            component_v_slot_style: None,
            default_v_slot_style: None,
            named_v_slot_style: None,
            v_bind_same_name_short_hand: None,
            strict_svelte_attr: false,
            svelte_attr_shorthand: None,
            svelte_directive_shorthand: None,
            astro_attr_shorthand: None,
        }
    }
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(rename_all = "kebab-case"))]
pub enum Quotes {
    #[default]
    Double,
    Single,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(rename_all = "kebab-case"))]
pub enum ClosingTagLineBreakForEmpty {
    Always,
    #[default]
    Fit,
    Never,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(rename_all = "kebab-case"))]
pub enum WhitespaceSensitivity {
    #[default]
    Css,
    Strict,
    Ignore,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(rename_all = "kebab-case"))]
pub enum VBindStyle {
    #[default]
    Short,
    Long,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(rename_all = "kebab-case"))]
pub enum VOnStyle {
    #[default]
    Short,
    Long,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(rename_all = "kebab-case"))]
pub enum VForDelimiterStyle {
    #[default]
    In,
    Of,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(rename_all = "kebab-case"))]
pub enum VSlotStyle {
    #[default]
    Short,
    Long,
    #[cfg_attr(feature = "config_serde", serde(alias = "vSlot", alias = "vslot"))]
    VSlot,
}
