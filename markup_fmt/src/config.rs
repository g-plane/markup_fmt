//! Types about configuration.
//!
//! For detailed documentation of configuration,
//! please read [configuration documentation](https://markup-fmt.netlify.app/).

#[cfg(feature = "config_serde")]
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(default))]
/// The whole configuration of markup_fmt.
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
    pub print_width: usize,

    #[cfg_attr(feature = "config_serde", serde(alias = "useTabs"))]
    pub use_tabs: bool,

    #[cfg_attr(feature = "config_serde", serde(alias = "indentWidth"))]
    pub indent_width: usize,

    #[cfg_attr(
        feature = "config_serde",
        serde(alias = "lineBreak", alias = "linebreak")
    )]
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
    pub quotes: Quotes,

    #[cfg_attr(feature = "config_serde", serde(alias = "formatComments"))]
    pub format_comments: bool,

    #[cfg_attr(feature = "config_serde", serde(alias = "scriptIndent"))]
    pub script_indent: bool,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "html.script_indent", alias = "html.scriptIndent")
    )]
    pub html_script_indent: Option<bool>,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "vue.script_indent", alias = "vue.scriptIndent")
    )]
    pub vue_script_indent: Option<bool>,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "svelte.script_indent", alias = "svelte.scriptIndent")
    )]
    pub svelte_script_indent: Option<bool>,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "astro.script_indent", alias = "astro.scriptIndent")
    )]
    pub astro_script_indent: Option<bool>,

    #[cfg_attr(feature = "config_serde", serde(alias = "styleIndent"))]
    pub style_indent: bool,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "html.style_indent", alias = "html.styleIndent")
    )]
    pub html_style_indent: Option<bool>,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "vue.style_indent", alias = "vue.styleIndent")
    )]
    pub vue_style_indent: Option<bool>,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "svelte.style_indent", alias = "svelte.styleIndent")
    )]
    pub svelte_style_indent: Option<bool>,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "astro.style_indent", alias = "astro.styleIndent")
    )]
    pub astro_style_indent: Option<bool>,

    #[cfg_attr(feature = "config_serde", serde(alias = "closingBracketSameLine"))]
    pub closing_bracket_same_line: bool,

    #[cfg_attr(feature = "config_serde", serde(alias = "closingTagLineBreakForEmpty"))]
    pub closing_tag_line_break_for_empty: ClosingTagLineBreakForEmpty,

    #[cfg_attr(feature = "config_serde", serde(alias = "maxAttrsPerLine"))]
    pub max_attrs_per_line: Option<NonZeroUsize>,

    #[cfg_attr(feature = "config_serde", serde(alias = "preferAttrsSingleLine"))]
    pub prefer_attrs_single_line: bool,

    #[cfg_attr(feature = "config_serde", serde(alias = "preferSingleLineOpeningTag"))]
    pub prefer_single_line_opening_tag: bool,

    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "html.normal.self_closing", alias = "html.normal.selfClosing")
    )]
    pub html_normal_self_closing: Option<bool>,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "html.void.self_closing", alias = "html.void.selfClosing")
    )]
    pub html_void_self_closing: Option<bool>,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "component.self_closing", alias = "component.selfClosing")
    )]
    pub component_self_closing: Option<bool>,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "svg.self_closing", alias = "svg.selfClosing")
    )]
    pub svg_self_closing: Option<bool>,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "mathml.self_closing", alias = "mathml.selfClosing")
    )]
    pub mathml_self_closing: Option<bool>,

    #[cfg_attr(feature = "config_serde", serde(alias = "whitespaceSensitivity"))]
    pub whitespace_sensitivity: WhitespaceSensitivity,
    #[cfg_attr(
        feature = "config_serde",
        serde(
            rename = "component.whitespace_sensitivity",
            alias = "component.whitespaceSensitivity"
        )
    )]
    pub component_whitespace_sensitivity: Option<WhitespaceSensitivity>,

    #[cfg_attr(feature = "config_serde", serde(alias = "doctypeKeywordCase"))]
    pub doctype_keyword_case: DoctypeKeywordCase,

    #[cfg_attr(feature = "config_serde", serde(alias = "vBindStyle"))]
    pub v_bind_style: Option<VBindStyle>,
    #[cfg_attr(feature = "config_serde", serde(alias = "vOnStyle"))]
    pub v_on_style: Option<VOnStyle>,
    #[cfg_attr(feature = "config_serde", serde(alias = "vForDelimiterStyle"))]
    pub v_for_delimiter_style: Option<VForDelimiterStyle>,
    #[cfg_attr(feature = "config_serde", serde(alias = "vSlotStyle"))]
    pub v_slot_style: Option<VSlotStyle>,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "component.v_slot_style", alias = "component.vSlotStyle")
    )]
    pub component_v_slot_style: Option<VSlotStyle>,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "default.v_slot_style", alias = "default.vSlotStyle")
    )]
    pub default_v_slot_style: Option<VSlotStyle>,
    #[cfg_attr(
        feature = "config_serde",
        serde(rename = "named.v_slot_style", alias = "named.vSlotStyle")
    )]
    pub named_v_slot_style: Option<VSlotStyle>,
    #[cfg_attr(feature = "config_serde", serde(alias = "vBindSameNameShortHand"))]
    pub v_bind_same_name_short_hand: Option<bool>,

    #[cfg_attr(feature = "config_serde", serde(alias = "strictSvelteAttr"))]
    pub strict_svelte_attr: bool,
    #[cfg_attr(feature = "config_serde", serde(alias = "svelteAttrShorthand"))]
    pub svelte_attr_shorthand: Option<bool>,
    #[cfg_attr(feature = "config_serde", serde(alias = "svelteDirectiveShorthand"))]
    pub svelte_directive_shorthand: Option<bool>,

    #[cfg_attr(feature = "config_serde", serde(alias = "astroAttrShorthand"))]
    pub astro_attr_shorthand: Option<bool>,

    #[cfg_attr(feature = "config_serde", serde(alias = "scriptFormatter"))]
    pub script_formatter: Option<ScriptFormatter>,

    #[cfg_attr(feature = "config_serde", serde(alias = "ignoreCommentDirective"))]
    pub ignore_comment_directive: String,

    #[cfg_attr(feature = "config_serde", serde(alias = "ignoreFileCommentDirective"))]
    pub ignore_file_comment_directive: String,
}

impl Default for LanguageOptions {
    fn default() -> Self {
        LanguageOptions {
            quotes: Quotes::default(),
            format_comments: false,
            script_indent: false,
            html_script_indent: None,
            vue_script_indent: None,
            svelte_script_indent: None,
            astro_script_indent: None,
            style_indent: false,
            html_style_indent: None,
            vue_style_indent: None,
            svelte_style_indent: None,
            astro_style_indent: None,
            closing_bracket_same_line: false,
            closing_tag_line_break_for_empty: ClosingTagLineBreakForEmpty::default(),
            max_attrs_per_line: None,
            prefer_attrs_single_line: false,
            prefer_single_line_opening_tag: false,
            html_normal_self_closing: None,
            html_void_self_closing: None,
            component_self_closing: None,
            svg_self_closing: None,
            mathml_self_closing: None,
            whitespace_sensitivity: WhitespaceSensitivity::default(),
            component_whitespace_sensitivity: None,
            doctype_keyword_case: DoctypeKeywordCase::default(),
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
            script_formatter: None,
            ignore_comment_directive: "markup-fmt-ignore".into(),
            ignore_file_comment_directive: "markup-fmt-ignore-file".into(),
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
pub enum DoctypeKeywordCase {
    Ignore,
    #[default]
    Upper,
    Lower,
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

#[derive(Clone, Debug)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(rename_all = "kebab-case"))]
pub enum ScriptFormatter {
    Dprint,
    Biome,
}
