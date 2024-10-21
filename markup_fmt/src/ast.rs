pub struct AngularCase<'s> {
    pub expr: (&'s str, usize),
    pub children: Vec<Node<'s>>,
}

pub struct AngularElseIf<'s> {
    pub expr: (&'s str, usize),
    pub reference: Option<(&'s str, usize)>,
    pub children: Vec<Node<'s>>,
}

pub struct AngularFor<'s> {
    pub binding: (&'s str, usize),
    pub expr: (&'s str, usize),
    pub track: Option<(&'s str, usize)>,
    pub aliases: Option<(&'s str, usize)>,
    pub children: Vec<Node<'s>>,
    pub empty: Option<Vec<Node<'s>>>,
}

pub struct AngularIf<'s> {
    pub expr: (&'s str, usize),
    pub reference: Option<(&'s str, usize)>,
    pub children: Vec<Node<'s>>,
    pub else_if_blocks: Vec<AngularElseIf<'s>>,
    pub else_children: Option<Vec<Node<'s>>>,
}

pub struct AngularInterpolation<'s> {
    pub expr: &'s str,
    pub start: usize,
}

pub struct AngularLet<'s> {
    pub name: &'s str,
    pub expr: (&'s str, usize),
}

pub struct AngularSwitch<'s> {
    pub expr: (&'s str, usize),
    pub cases: Vec<AngularCase<'s>>,
    pub default: Option<Vec<Node<'s>>>,
}

pub struct AstroAttribute<'s> {
    pub name: Option<&'s str>,
    pub expr: (&'s str, usize),
}

pub struct AstroExpr<'s> {
    pub children: Vec<AstroExprChild<'s>>,
    pub has_line_comment: bool,
    pub start: usize,
}

pub enum AstroExprChild<'s> {
    Script(&'s str),
    Template(Vec<Node<'s>>),
}

pub enum Attribute<'s> {
    Astro(AstroAttribute<'s>),
    JinjaBlock(JinjaBlock<'s, Attribute<'s>>),
    JinjaTag(JinjaTag<'s>),
    Native(NativeAttribute<'s>),
    Svelte(SvelteAttribute<'s>),
    VentoTagOrBlock(NodeKind<'s>),
    VueDirective(VueDirective<'s>),
}

pub struct Comment<'s> {
    pub raw: &'s str,
}

pub struct Doctype<'s> {
    pub keyword: &'s str,
    pub value: &'s str,
}

pub struct Element<'s> {
    pub tag_name: &'s str,
    pub attrs: Vec<Attribute<'s>>,
    pub first_attr_same_line: bool,
    pub children: Vec<Node<'s>>,
    pub self_closing: bool,
    pub void_element: bool,
}

pub struct FrontMatter<'s> {
    pub raw: &'s str,
    pub start: usize,
}

pub struct JinjaBlock<'s, T> {
    pub body: Vec<JinjaTagOrChildren<'s, T>>,
}

pub struct JinjaComment<'s> {
    pub raw: &'s str,
}

pub struct JinjaInterpolation<'s> {
    pub expr: &'s str,
}

pub struct JinjaTag<'s> {
    pub content: &'s str,
}

pub enum JinjaTagOrChildren<'s, T> {
    Tag(JinjaTag<'s>),
    Children(Vec<T>),
}

pub struct NativeAttribute<'s> {
    pub name: &'s str,
    pub value: Option<(&'s str, usize)>,
    pub quote: Option<char>,
}

pub struct Node<'s> {
    pub kind: NodeKind<'s>,
    pub raw: &'s str,
}

pub enum NodeKind<'s> {
    AngularFor(AngularFor<'s>),
    AngularIf(AngularIf<'s>),
    AngularInterpolation(AngularInterpolation<'s>),
    AngularLet(AngularLet<'s>),
    AngularSwitch(AngularSwitch<'s>),
    AstroExpr(AstroExpr<'s>),
    Comment(Comment<'s>),
    Doctype(Doctype<'s>),
    Element(Element<'s>),
    FrontMatter(FrontMatter<'s>),
    JinjaBlock(JinjaBlock<'s, Node<'s>>),
    JinjaComment(JinjaComment<'s>),
    JinjaInterpolation(JinjaInterpolation<'s>),
    JinjaTag(JinjaTag<'s>),
    SvelteAtTag(SvelteAtTag<'s>),
    SvelteAwaitBlock(Box<SvelteAwaitBlock<'s>>),
    SvelteEachBlock(SvelteEachBlock<'s>),
    SvelteIfBlock(SvelteIfBlock<'s>),
    SvelteInterpolation(SvelteInterpolation<'s>),
    SvelteKeyBlock(SvelteKeyBlock<'s>),
    Text(TextNode<'s>),
    VentoBlock(VentoBlock<'s>),
    VentoComment(VentoComment<'s>),
    VentoEval(VentoEval<'s>),
    VentoInterpolation(VentoInterpolation<'s>),
    VentoTag(VentoTag<'s>),
    VueInterpolation(VueInterpolation<'s>),
}

pub struct Root<'s> {
    pub children: Vec<Node<'s>>,
}

pub struct SvelteAtTag<'s> {
    pub name: &'s str,
    pub expr: (&'s str, usize),
}

pub struct SvelteAttribute<'s> {
    pub name: Option<&'s str>,
    pub expr: (&'s str, usize),
}

pub struct SvelteAwaitBlock<'s> {
    pub expr: (&'s str, usize),
    pub then_binding: Option<(&'s str, usize)>,
    pub catch_binding: Option<(&'s str, usize)>,
    pub children: Vec<Node<'s>>,
    pub then_block: Option<SvelteThenBlock<'s>>,
    pub catch_block: Option<SvelteCatchBlock<'s>>,
}

pub struct SvelteCatchBlock<'s> {
    pub binding: Option<(&'s str, usize)>,
    pub children: Vec<Node<'s>>,
}

pub struct SvelteEachBlock<'s> {
    pub expr: (&'s str, usize),
    pub binding: (&'s str, usize),
    pub index: Option<&'s str>,
    pub key: Option<(&'s str, usize)>,
    pub children: Vec<Node<'s>>,
    pub else_children: Option<Vec<Node<'s>>>,
}

pub struct SvelteElseIfBlock<'s> {
    pub expr: (&'s str, usize),
    pub children: Vec<Node<'s>>,
}

pub struct SvelteIfBlock<'s> {
    pub expr: (&'s str, usize),
    pub children: Vec<Node<'s>>,
    pub else_if_blocks: Vec<SvelteElseIfBlock<'s>>,
    pub else_children: Option<Vec<Node<'s>>>,
}

pub struct SvelteInterpolation<'s> {
    pub expr: (&'s str, usize),
}

pub struct SvelteKeyBlock<'s> {
    pub expr: (&'s str, usize),
    pub children: Vec<Node<'s>>,
}

pub struct SvelteThenBlock<'s> {
    pub binding: (&'s str, usize),
    pub children: Vec<Node<'s>>,
}

pub struct TextNode<'s> {
    pub raw: &'s str,
    pub line_breaks: usize,
    pub start: usize,
}

pub struct VentoBlock<'s> {
    pub body: Vec<VentoTagOrChildren<'s>>,
}

pub struct VentoComment<'s> {
    pub raw: &'s str,
}

pub struct VentoEval<'s> {
    pub raw: &'s str,
    pub start: usize,
}

pub struct VentoInterpolation<'s> {
    pub expr: &'s str,
    pub start: usize,
}

pub struct VentoTag<'s> {
    pub tag: &'s str,
    pub trim_prev: bool,
    pub trim_next: bool,
}

pub enum VentoTagOrChildren<'s> {
    Tag(VentoTag<'s>),
    Children(Vec<Node<'s>>),
}

pub struct VueDirective<'s> {
    pub name: &'s str,
    pub arg_and_modifiers: Option<&'s str>,
    pub value: Option<(&'s str, usize)>,
}

pub struct VueInterpolation<'s> {
    pub expr: &'s str,
    pub start: usize,
}
