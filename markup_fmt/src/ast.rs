#[derive(Clone, Debug)]
pub struct AstroAttribute<'s> {
    pub name: Option<&'s str>,
    pub expr: &'s str,
}

#[derive(Clone, Debug)]
pub struct AstroInterpolation<'s> {
    pub expr: &'s str,
}

#[derive(Clone, Debug)]
pub struct AstroScriptBlock<'s> {
    pub raw: &'s str,
}

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug)]
pub enum Attribute<'s> {
    AstroAttribute(AstroAttribute<'s>),
    NativeAttribute(NativeAttribute<'s>),
    SvelteAttribute(SvelteAttribute<'s>),
    VueDirective(VueDirective<'s>),
}

#[derive(Clone, Debug)]
pub struct Comment<'s> {
    pub raw: &'s str,
}

#[derive(Clone, Debug)]
pub struct VueDirective<'s> {
    pub name: &'s str,
    pub arg_and_modifiers: Option<&'s str>,
    pub value: Option<&'s str>,
}

#[derive(Clone, Debug)]
pub struct Element<'s> {
    pub tag_name: &'s str,
    pub attrs: Vec<Attribute<'s>>,
    pub children: Vec<Node<'s>>,
    pub self_closing: bool,
    pub void_element: bool,
}

#[derive(Clone, Debug)]
pub struct JinjaBlock<'s> {
    pub body: Vec<JinjaTagOrChildren<'s>>,
}

#[derive(Clone, Debug)]
pub struct JinjaComment<'s> {
    pub raw: &'s str,
}

#[derive(Clone, Debug)]
pub struct JinjaInterpolation<'s> {
    pub expr: &'s str,
}

#[derive(Clone, Debug)]
pub struct JinjaTag<'s> {
    pub content: &'s str,
}

#[derive(Clone, Debug)]
pub enum JinjaTagOrChildren<'s> {
    Tag(JinjaTag<'s>),
    Children(Vec<Node<'s>>),
}

#[derive(Clone, Debug)]
pub struct NativeAttribute<'s> {
    pub name: &'s str,
    pub value: Option<&'s str>,
}

#[derive(Clone, Debug)]
pub enum Node<'s> {
    AstroInterpolation(AstroInterpolation<'s>),
    AstroScriptBlock(AstroScriptBlock<'s>),
    Comment(Comment<'s>),
    Doctype,
    Element(Element<'s>),
    JinjaBlock(JinjaBlock<'s>),
    JinjaComment(JinjaComment<'s>),
    JinjaInterpolation(JinjaInterpolation<'s>),
    JinjaTag(JinjaTag<'s>),
    SvelteAtTag(SvelteAtTag<'s>),
    SvelteAwaitBlock(Box<SvelteAwaitBlock<'s>>),
    SvelteEachBlock(SvelteEachBlock<'s>),
    SvelteIfBlock(SvelteIfBlock<'s>),
    SvelteInterpolation(SvelteInterpolation<'s>),
    SvelteKeyBlock(SvelteKeyBlock<'s>),
    TextNode(TextNode<'s>),
    VueInterpolation(VueInterpolation<'s>),
}

#[derive(Clone, Debug)]
pub struct Root<'s> {
    pub children: Vec<Node<'s>>,
}

#[derive(Clone, Debug)]
pub struct SvelteAtTag<'s> {
    pub name: &'s str,
    pub expr: &'s str,
}

#[derive(Clone, Debug)]
pub struct SvelteAttribute<'s> {
    pub name: Option<&'s str>,
    pub expr: &'s str,
}

#[derive(Clone, Debug)]
pub struct SvelteAwaitBlock<'s> {
    pub expr: &'s str,
    pub then_binding: Option<&'s str>,
    pub catch_binding: Option<&'s str>,
    pub children: Vec<Node<'s>>,
    pub then_block: Option<SvelteThenBlock<'s>>,
    pub catch_block: Option<SvelteCatchBlock<'s>>,
}

#[derive(Clone, Debug)]
pub struct SvelteCatchBlock<'s> {
    pub binding: Option<&'s str>,
    pub children: Vec<Node<'s>>,
}

#[derive(Clone, Debug)]
pub struct SvelteEachBlock<'s> {
    pub expr: &'s str,
    pub binding: &'s str,
    pub index: Option<&'s str>,
    pub key: Option<&'s str>,
    pub children: Vec<Node<'s>>,
    pub else_children: Option<Vec<Node<'s>>>,
}

#[derive(Clone, Debug)]
pub struct SvelteElseIfBlock<'s> {
    pub expr: &'s str,
    pub children: Vec<Node<'s>>,
}

#[derive(Clone, Debug)]
pub struct SvelteIfBlock<'s> {
    pub expr: &'s str,
    pub children: Vec<Node<'s>>,
    pub else_if_blocks: Vec<SvelteElseIfBlock<'s>>,
    pub else_children: Option<Vec<Node<'s>>>,
}

#[derive(Clone, Debug)]
pub struct SvelteInterpolation<'s> {
    pub expr: &'s str,
}

#[derive(Clone, Debug)]
pub struct SvelteKeyBlock<'s> {
    pub expr: &'s str,
    pub children: Vec<Node<'s>>,
}

#[derive(Clone, Debug)]
pub struct SvelteThenBlock<'s> {
    pub binding: &'s str,
    pub children: Vec<Node<'s>>,
}

#[derive(Clone, Debug)]
pub struct TextNode<'s> {
    pub raw: &'s str,
    pub line_breaks: usize,
}

#[derive(Clone, Debug)]
pub struct VueInterpolation<'s> {
    pub expr: &'s str,
}
