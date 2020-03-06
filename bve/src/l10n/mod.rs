use crate::data::DATA;
use fluent::{FluentBundle, FluentResource};
use include_dir::File;
use once_cell::sync::Lazy;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::sync::Mutex;
use unic_langid::{langid, LanguageIdentifier};

pub static CURRENT_BVE_LOCALE: Lazy<Mutex<BVELocale>> = Lazy::new(|| Mutex::new(load_locale(Language::EN)));

pub struct BVELocale {
    language: Language,
    strings: FluentBundle<FluentResource>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Language {
    EN,
    DE,
}

impl Language {
    #[must_use]
    pub fn get_identifier(self) -> LanguageIdentifier {
        match self {
            Self::EN => langid!("en-US"),
            Self::DE => langid!("de-DE"),
        }
    }
}

impl Display for Language {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::EN => write!(f, "us"),
            Self::DE => write!(f, "de"),
        }
    }
}

pub fn load_locale(language: Language) -> BVELocale {
    unimplemented!()
}

#[must_use]
pub fn enumerate_language_files(language: Language) -> &'static [File<'static>] {
    DATA.get_dir(format!("l10n/{}", language))
        .unwrap_or_else(|| panic!("Missing language dir {}", language))
        .files()
}
