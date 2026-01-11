#[derive(Debug)]
/// Angular for loop: `@for ( ... )`.
///
/// See https://angular.dev/api/core/@for.
pub struct AngularFor<'s> {
    pub binding: (&'s str, usize),
    pub expr: (&'s str, usize),
    pub track: Option<(&'s str, usize)>,
    pub aliases: Vec<(&'s str, usize)>,
    pub children: Vec<Node<'s>>,
    pub empty: Option<Vec<Node<'s>>>,
}

#[derive(Debug)]
/// Angular conditional block: `@if ( condition )`.
///
/// See https://angular.dev/api/core/@if.
pub struct AngularIf<'s> {
    pub expr: (&'s str, usize),
    pub reference: Option<(&'s str, usize)>,
    pub children: Vec<Node<'s>>,
    pub else_if_blocks: Vec<AngularElseIf<'s>>,
    pub else_children: Option<Vec<Node<'s>>>,
}

#[derive(Debug)]
/// Angular else-if block: `@else if ( condition )`.
///
/// See https://angular.dev/api/core/@if.
pub struct AngularElseIf<'s> {
    pub expr: (&'s str, usize),
    pub reference: Option<(&'s str, usize)>,
    pub children: Vec<Node<'s>>,
}

#[derive(Debug)]
pub struct AngularGenericBlock<'s> {
    pub keyword: &'s str,
    pub header: Option<&'s str>,
    pub children: Vec<Node<'s>>,
}

#[derive(Debug)]
/// Angular interpolation: `{{ expression }}`.
///
/// See https://angular.dev/guide/templates/binding#render-dynamic-text-with-text-interpolation.
pub struct AngularInterpolation<'s> {
    pub expr: &'s str,
    pub start: usize,
}

#[derive(Debug)]
/// Angular let variable declaration: `@let name = expression`.
///
/// See https://angular.dev/api/core/@let.
pub struct AngularLet<'s> {
    pub name: &'s str,
    pub expr: (&'s str, usize),
}

#[derive(Debug)]
/// Angular switch statement: `@switch (expression)`.
///
/// See https://angular.dev/api/core/@switch.
pub struct AngularSwitch<'s> {
    pub expr: (&'s str, usize),
    pub arms: Vec<AngularSwitchArm<'s>>,
}

#[derive(Debug)]
/// `@case` or `@default` arm of an `AngularSwitch`.
///
/// See https://angular.dev/api/core/@switch.
pub struct AngularSwitchArm<'s> {
    pub keyword: &'static str,
    pub expr: Option<(&'s str, usize)>,
    pub children: Vec<Node<'s>>,
}

#[derive(Debug)]
/// Astro attribute: `{expression}` or `name={expression}`.
///
/// See https://docs.astro.build/en/reference/astro-syntax/#dynamic-attributes.
pub struct AstroAttribute<'s> {
    pub name: Option<&'s str>,
    pub expr: (&'s str, usize),
}

#[derive(Debug)]
/// Astro expression block: `{...}`.
///
/// See https://docs.astro.build/en/reference/astro-syntax/#dynamic-html.
pub struct AstroExpr<'s> {
    pub children: Vec<AstroExprChild<'s>>,
    pub has_line_comment: bool,
    pub start: usize,
}

#[derive(Debug)]
/// See https://docs.astro.build/en/core-concepts/astro-syntax/#dynamic-html.
pub enum AstroExprChild<'s> {
    Script(&'s str),
    Template(Vec<Node<'s>>),
}

#[derive(Debug)]
pub enum Attribute<'s> {
    Astro(AstroAttribute<'s>),
    JinjaBlock(JinjaBlock<'s, Attribute<'s>>),
    JinjaComment(JinjaComment<'s>),
    JinjaTag(JinjaTag<'s>),
    Native(NativeAttribute<'s>),
    Svelte(SvelteAttribute<'s>),
    SvelteAttachment(SvelteAttachment<'s>),
    VentoTagOrBlock(NodeKind<'s>),
    VueDirective(VueDirective<'s>),
}

#[derive(Debug)]
/// `<![CDATA[ ... ]]>`
///
/// See https://www.w3.org/TR/xml/#sec-cdata-sect
pub struct Cdata<'s> {
    pub raw: &'s str,
}

#[derive(Debug)]
/// Comment in HTML: `<!-- ... -->`.
///
/// See https://developer.mozilla.org/en-US/docs/Web/HTML/Comments
pub struct Comment<'s> {
    pub raw: &'s str,
}

#[derive(Debug)]
/// HTML doctype declaration: `<!DOCTYPE ...>`.
///
/// See https://developer.mozilla.org/en-US/docs/Glossary/Doctype
pub struct Doctype<'s> {
    pub keyword: &'s str,
    pub value: &'s str,
}

#[derive(Debug)]
/// HTML element with its attributes and children.
///
/// See https://developer.mozilla.org/en-US/docs/Web/HTML/Element
pub struct Element<'s> {
    pub tag_name: &'s str,
    pub attrs: Vec<Attribute<'s>>,
    pub first_attr_same_line: bool,
    pub children: Vec<Node<'s>>,
    pub self_closing: bool,
    pub void_element: bool,
}

#[derive(Debug)]
/// Front matter content in a file, typically enclosed in `---`.
///
/// See https://docs.astro.build/en/guides/markdown-content/.
pub struct FrontMatter<'s> {
    pub raw: &'s str,
    pub start: usize,
}

#[derive(Debug)]
/// Jinja block containing nested Jinja tags or HTML elements.
///
/// See https://jinja.palletsprojects.com/en/stable/templates/#list-of-control-structures.
pub struct JinjaBlock<'s, T> {
    pub body: Vec<JinjaTagOrChildren<'s, T>>,
}

#[derive(Debug)]
/// Jinja comment: `{# ... #}`.
///
/// See https://jinja.palletsprojects.com/en/stable/templates/#comments.
pub struct JinjaComment<'s> {
    pub raw: &'s str,
}

#[derive(Debug)]
/// Jinja interpolation: `{{ ... }}`.
///
/// See https://jinja.palletsprojects.com/en/stable/templates/#expressions.
pub struct JinjaInterpolation<'s> {
    pub expr: &'s str,
    pub start: usize,
    pub trim_prev: bool,
    pub trim_next: bool,
}

#[derive(Debug)]
/// Jinja tag: `{% ... %}`.
///
/// See https://jinja.palletsprojects.com/en/stable/templates/#list-of-control-structures.
pub struct JinjaTag<'s> {
    pub content: &'s str,
    pub start: usize,
}

#[derive(Debug)]
pub enum JinjaTagOrChildren<'s, T> {
    Tag(JinjaTag<'s>),
    Children(Vec<T>),
}

#[derive(Debug)]
/// Mustache block: `{{#variable}}{{/variable}}`.
///
/// See https://mustache.github.io/mustache.5.html
pub struct MustacheBlock<'s> {
    pub controls: Vec<MustacheBlockControl<'s>>,
    pub children: Vec<Vec<Node<'s>>>,
}

#[derive(Debug)]
pub struct MustacheBlockControl<'s> {
    pub name: &'s str,
    pub prefix: &'s str,
    pub content: Option<&'s str>,
    pub wc_before: bool,
    pub wc_after: bool,
}

#[derive(Debug)]
/// Mustache interpolation: `{{expression}}`.
///
/// See https://mustache.github.io/mustache.5.html
pub struct MustacheInterpolation<'s> {
    pub content: &'s str,
}

#[derive(Debug)]
/// Standard HTML attribute.
///
/// See https://developer.mozilla.org/en-US/docs/Glossary/Attribute
pub struct NativeAttribute<'s> {
    pub name: &'s str,
    pub value: Option<(&'s str, usize)>,
    pub quote: Option<char>,
}

#[derive(Debug)]
pub struct Node<'s> {
    pub kind: NodeKind<'s>,
    pub raw: &'s str,
}

#[derive(Debug)]
pub enum NodeKind<'s> {
    AngularFor(AngularFor<'s>),
    AngularGenericBlocks(Vec<AngularGenericBlock<'s>>),
    AngularIf(AngularIf<'s>),
    AngularInterpolation(AngularInterpolation<'s>),
    AngularLet(AngularLet<'s>),
    AngularSwitch(AngularSwitch<'s>),
    AstroExpr(AstroExpr<'s>),
    Cdata(Cdata<'s>),
    Comment(Comment<'s>),
    Doctype(Doctype<'s>),
    Element(Element<'s>),
    FrontMatter(FrontMatter<'s>),
    JinjaBlock(JinjaBlock<'s, Node<'s>>),
    JinjaComment(JinjaComment<'s>),
    JinjaInterpolation(JinjaInterpolation<'s>),
    JinjaTag(JinjaTag<'s>),
    MustacheBlock(MustacheBlock<'s>),
    MustacheInterpolation(MustacheInterpolation<'s>),
    SvelteAtTag(SvelteAtTag<'s>),
    SvelteAwaitBlock(Box<SvelteAwaitBlock<'s>>),
    SvelteEachBlock(SvelteEachBlock<'s>),
    SvelteIfBlock(SvelteIfBlock<'s>),
    SvelteInterpolation(SvelteInterpolation<'s>),
    SvelteKeyBlock(SvelteKeyBlock<'s>),
    SvelteSnippetBlock(SvelteSnippetBlock<'s>),
    Text(TextNode<'s>),
    VentoBlock(VentoBlock<'s>),
    VentoComment(VentoComment<'s>),
    VentoEval(VentoEval<'s>),
    VentoInterpolation(VentoInterpolation<'s>),
    VentoTag(VentoTag<'s>),
    VueInterpolation(VueInterpolation<'s>),
    XmlDecl(XmlDecl<'s>),
}

#[derive(Debug)]
pub struct Root<'s> {
    pub children: Vec<Node<'s>>,
}

#[derive(Debug)]
/// Svelte `@` tag: (`@render`, `@const`, etc).
///
/// See https://svelte.dev/docs/svelte/@render.
pub struct SvelteAtTag<'s> {
    pub name: &'s str,
    pub expr: (&'s str, usize),
}

#[derive(Debug)]
/// Svelte attribute: `{expression}` or `name={expression}`.
///
/// See https://svelte.dev/docs/svelte/basic-markup#Element-attributes.
pub struct SvelteAttribute<'s> {
    pub name: Option<&'s str>,
    pub expr: (&'s str, usize),
}

#[derive(Debug)]
/// Svelte attachment: `{@attach expression}`.
///
/// See https://svelte.dev/docs/svelte/@attach.
pub struct SvelteAttachment<'s> {
    pub expr: (&'s str, usize),
}

#[derive(Debug)]
/// Svelte await block `{#await expression}...{:then name}...{:catch name}...{/await}`.
///
/// See https://svelte.dev/docs/svelte/await.
pub struct SvelteAwaitBlock<'s> {
    pub expr: (&'s str, usize),
    pub then_binding: Option<Option<(&'s str, usize)>>, // binding can be optional with `then` keyword only
    pub catch_binding: Option<Option<(&'s str, usize)>>, // binding can be optional with `catch` keyword only
    pub children: Vec<Node<'s>>,
    pub then_block: Option<SvelteThenBlock<'s>>,
    pub catch_block: Option<SvelteCatchBlock<'s>>,
}

#[derive(Debug)]
/// The `{:catch error}...` part of a `SvelteAwaitBlock`.
pub struct SvelteCatchBlock<'s> {
    pub binding: Option<(&'s str, usize)>,
    pub children: Vec<Node<'s>>,
}

#[derive(Debug)]
/// The `{:then value}...` part of a `SvelteAwaitBlock`.
pub struct SvelteThenBlock<'s> {
    pub binding: Option<(&'s str, usize)>,
    pub children: Vec<Node<'s>>,
}

#[derive(Debug)]
/// Svelte each block: `{#each expression as name}...{/each}`.
///
/// See https://svelte.dev/docs/svelte/each.
pub struct SvelteEachBlock<'s> {
    pub expr: (&'s str, usize),
    pub binding: Option<(&'s str, usize)>,
    pub index: Option<&'s str>,
    pub key: Option<(&'s str, usize)>,
    pub children: Vec<Node<'s>>,
    pub else_children: Option<Vec<Node<'s>>>,
}

#[derive(Debug)]
/// Svelte if block: `{#if expression}...{:else if expression}...{/if}`.
///
/// See https://svelte.dev/docs/svelte/if.
pub struct SvelteIfBlock<'s> {
    pub expr: (&'s str, usize),
    pub children: Vec<Node<'s>>,
    pub else_if_blocks: Vec<SvelteElseIfBlock<'s>>,
    pub else_children: Option<Vec<Node<'s>>>,
}

#[derive(Debug)]
/// The `{:else if condition}...` part of a `SvelteIfBlock`.
pub struct SvelteElseIfBlock<'s> {
    pub expr: (&'s str, usize),
    pub children: Vec<Node<'s>>,
}

#[derive(Debug)]
/// Svelte interpolation: `{expression}`.
///
/// See https://svelte.dev/docs/svelte/basic-markup#Text-expressions.
pub struct SvelteInterpolation<'s> {
    pub expr: (&'s str, usize),
}

#[derive(Debug)]
/// Svelte key block: `{#key expression}...{/key}`.
///
/// See https://svelte.dev/docs/svelte/key.
pub struct SvelteKeyBlock<'s> {
    pub expr: (&'s str, usize),
    pub children: Vec<Node<'s>>,
}

#[derive(Debug)]
/// Svelte snippet block: `{#snippet name()}...{/snippet}`.
///
/// See https://svelte.dev/docs/svelte/snippet.
pub struct SvelteSnippetBlock<'s> {
    pub signature: (&'s str, usize),
    pub children: Vec<Node<'s>>,
}

#[derive(Debug)]
/// Plain text node.
pub struct TextNode<'s> {
    pub raw: &'s str,
    pub line_breaks: usize,
    pub start: usize,
}

#[derive(Debug)]
/// Vento block: `{{ keyword ... }}...{{ /keyword }}`
///
/// See https://vento.js.org/syntax/blocks.
pub struct VentoBlock<'s> {
    pub body: Vec<VentoTagOrChildren<'s>>,
}

#[derive(Debug)]
/// Vento comment: `{{# ... #}}`.
///
/// See https://vento.js.org/syntax/comments/.
pub struct VentoComment<'s> {
    pub raw: &'s str,
}

#[derive(Debug)]
/// Vento eval block for JavaScript evaluation: `{{> ... }}`.
///
/// See https://vento.js.org/syntax/javascript/.
pub struct VentoEval<'s> {
    pub raw: &'s str,
    pub start: usize,
}

#[derive(Debug)]
/// Vento interpolation `{{ ... }}`.
///
/// See https://vento.js.org/syntax/print/.
pub struct VentoInterpolation<'s> {
    pub expr: &'s str,
    pub start: usize,
    pub trim_prev: bool,
    pub trim_next: bool,
}

#[derive(Debug)]
/// Vento tag: `{{ keyword ... }}`.
///
/// See https://vento.js.org/syntax/include/.
pub struct VentoTag<'s> {
    pub tag: &'s str,
    pub trim_prev: bool,
    pub trim_next: bool,
}

#[derive(Debug)]
pub enum VentoTagOrChildren<'s> {
    Tag(VentoTag<'s>),
    Children(Vec<Node<'s>>),
}

#[derive(Debug)]
/// Vue directive: `v-if`, `v-for`, etc.
///
/// See https://vuejs.org/guide/essentials/template-syntax.html#directives.
pub struct VueDirective<'s> {
    pub name: &'s str,
    pub arg_and_modifiers: Option<&'s str>,
    pub value: Option<(&'s str, usize)>,
}

#[derive(Debug)]
/// Vue interpolation: `{{ expression }}`.
///
/// See https://vuejs.org/guide/essentials/template-syntax.html#text-interpolation.
pub struct VueInterpolation<'s> {
    pub expr: &'s str,
    pub start: usize,
}

#[derive(Debug)]
/// XML declaration.
///
/// See https://www.w3.org/TR/xml/#sec-prolog-dtd
pub struct XmlDecl<'s> {
    pub attrs: Vec<NativeAttribute<'s>>,
}
