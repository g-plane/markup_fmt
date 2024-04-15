#[derive(Clone)]
pub(crate) struct State<'s> {
    pub(crate) current_tag_name: Option<&'s str>,
    pub(crate) is_root: bool,
    pub(crate) in_svg: bool,
}
