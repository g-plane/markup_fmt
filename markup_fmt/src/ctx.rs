use crate::{config::LanguageOptions, Language};

pub(crate) struct Ctx {
    pub(crate) language: Language,
    pub(crate) indent_width: usize,
    pub(crate) options: LanguageOptions,
}
