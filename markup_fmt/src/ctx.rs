use crate::{config::LanguageOptions, Language};
use std::{borrow::Cow, path::Path};

pub(crate) struct Ctx<'b, F>
where
    F: for<'a> FnMut(&Path, &'a str) -> Cow<'a, str>,
{
    pub(crate) language: Language,
    pub(crate) indent_width: usize,
    pub(crate) options: &'b LanguageOptions,
    pub(crate) external_formatter: F,
}

impl<'b, F> Ctx<'b, F>
where
    F: for<'a> FnMut(&Path, &'a str) -> Cow<'a, str>,
{
    pub(crate) fn format_with_external_formatter<'a>(
        &mut self,
        path: &Path,
        code: &'a str,
    ) -> Cow<'a, str> {
        (self.external_formatter)(path, code)
    }
}
