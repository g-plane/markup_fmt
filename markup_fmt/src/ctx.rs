use crate::{config::LanguageOptions, Language};
use std::{borrow::Cow, path::Path};

pub(crate) struct Ctx<'b, E, F>
where
    F: for<'a> FnMut(&Path, &'a str) -> Result<Cow<'a, str>, E>,
{
    pub(crate) language: Language,
    pub(crate) indent_width: usize,
    pub(crate) options: &'b LanguageOptions,
    pub(crate) external_formatter: F,
    pub(crate) external_formatter_error: Option<E>,
}

impl<'b, E, F> Ctx<'b, E, F>
where
    F: for<'a> FnMut(&Path, &'a str) -> Result<Cow<'a, str>, E>,
{
    pub(crate) fn format_with_external_formatter<'a>(
        &mut self,
        path: &Path,
        code: &'a str,
    ) -> Cow<'a, str> {
        match (self.external_formatter)(path, code) {
            Ok(code) => code,
            Err(e) => {
                self.external_formatter_error = Some(e);
                code.into()
            }
        }
    }
}
