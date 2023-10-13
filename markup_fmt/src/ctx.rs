use crate::{config::LanguageOptions, Language};

pub(crate) struct Ctx<'s> {
    pub(crate) source: &'s str,
    pub(crate) language: Language,
    pub(crate) indent_width: usize,
    pub(crate) options: LanguageOptions,
}
