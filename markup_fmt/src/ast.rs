#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug)]
pub enum Attribute<'s> {
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
pub struct NativeAttribute<'s> {
    pub name: &'s str,
    pub value: Option<&'s str>,
}

#[derive(Clone, Debug)]
pub enum Node<'s> {
    Comment(Comment<'s>),
    Element(Element<'s>),
    SvelteIfBlock(SvelteIfBlock<'s>),
    SvelteInterpolation(SvelteInterpolation<'s>),
    TextNode(TextNode<'s>),
    VueInterpolation(VueInterpolation<'s>),
}

#[derive(Clone, Debug)]
pub struct Root<'s> {
    pub children: Vec<Node<'s>>,
}

#[derive(Clone, Debug)]
pub struct SvelteAttribute<'s> {
    pub name: &'s str,
    pub expr: &'s str,
}

#[derive(Clone, Debug)]
pub struct SvelteIfBlock<'s> {
    pub expr: &'s str,
    pub children: Vec<Node<'s>>,
    pub else_children: Option<Vec<Node<'s>>>,
}

#[derive(Clone, Debug)]
pub struct SvelteInterpolation<'s> {
    pub expr: &'s str,
}

#[derive(Clone, Debug)]
pub struct TextNode<'s> {
    pub raw: &'s str,
}

#[derive(Clone, Debug)]
pub struct VueInterpolation<'s> {
    pub expr: &'s str,
}
