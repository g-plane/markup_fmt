use crate::{config::LanguageOptions, Language};
use std::{borrow::Cow, path::Path};

pub(crate) struct Ctx<F>
where
    F: for<'a> Fn(&Path, &'a str) -> Cow<'a, str>,
{
    pub(crate) language: Language,
    pub(crate) indent_width: usize,
    pub(crate) options: LanguageOptions,
    pub(crate) external_formatter: F,
}

impl<F> Ctx<F>
where
    F: for<'a> Fn(&Path, &'a str) -> Cow<'a, str>,
{
    pub(crate) fn format_with_external_formatter<'a>(
        &self,
        path: &Path,
        code: &'a str,
    ) -> Cow<'a, str> {
        (self.external_formatter)(path, code)
    }
}
