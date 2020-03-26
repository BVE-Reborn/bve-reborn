pub use current::*;
use fluent::{FluentBundle, FluentResource};
pub use load::*;
use once_cell::sync::Lazy;
use std::{
    fmt,
    fmt::{Display, Formatter},
    sync::RwLock,
};
use unic_langid::{langid, LanguageIdentifier};

mod current;
mod load;

pub static CURRENT_BVE_LOCALE: Lazy<RwLock<BVELocaleBundle>> =
    Lazy::new(|| RwLock::new(load_locale_bundle(get_current_locale())));
pub static ENGLISH_LOCALE: Lazy<BVELocaleBundle> =
    Lazy::new(|| load_locale_bundle(BVELocale::from_language(BVELanguage::EN)));

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ForceEnglish {
    English,
    Local,
}

pub struct BVELocaleBundle {
    pub language: BVELocale,
    pub bundle: FluentBundle<FluentResource>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BVELocale {
    pub langid: LanguageIdentifier,
    pub lang: BVELanguage,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BVELanguage {
    EN,
    DE,
}

impl BVELocale {
    #[must_use]
    pub fn from_ident(langid: LanguageIdentifier) -> Self {
        let lang = match langid.language() {
            "de" => BVELanguage::DE,
            _ => BVELanguage::EN,
        };
        Self { langid, lang }
    }

    #[must_use]
    pub fn from_language(lang: BVELanguage) -> Self {
        let langid = match lang {
            BVELanguage::EN => langid!("en-US"),
            BVELanguage::DE => langid!("de-DE"),
        };
        Self { langid, lang }
    }

    #[must_use]
    pub fn to_ident(&self) -> &'static str {
        self.lang.to_ident()
    }
}

impl BVELanguage {
    #[must_use]
    pub fn to_ident(self) -> &'static str {
        match self {
            Self::EN => "en",
            Self::DE => "de",
        }
    }
}

impl Display for BVELanguage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_ident())
    }
}

#[macro_export]
macro_rules! localize {
    // Localize in english for logging
    (english, $name:literal, $($key:literal -> $value:expr),+ $(,)*) => {
        $crate::localize!($crate::l10n::ENGLISH_LOCALE, $name, $($key -> $value),+)
    };
    (english, $name:literal) => {
        $crate::localize!($crate::l10n::ENGLISH_LOCALE, $name)
    };
    (@$cond:expr, $name:literal, $($key:literal -> $value:expr),+ $(,)*) => {
        if $cond == $crate::l10n::ForceEnglish::English {
            $crate::localize!($crate::l10n::ENGLISH_LOCALE, $name, $($key -> $value),+)
        } else {
            $crate::localize!($name, $($key -> $value),+)
        }
    };
    (@$cond:expr, $name:literal) => {
        if $cond == $crate::l10n::ForceEnglish::English {
            $crate::localize!($crate::l10n::ENGLISH_LOCALE, $name)
        } else {
            $crate::localize!($name)
        }
    };
    // Localize in the current locale
    ($name:literal, $($key:literal -> $value:expr),+ $(,)*) => {
        $crate::localize!($crate::l10n::CURRENT_BVE_LOCALE.read().expect("Unable to lock LocaleMutex"), $name, $($key -> $value),+)
    };
    ($name:literal) => {
        $crate::localize!($crate::l10n::CURRENT_BVE_LOCALE.read().expect("Unable to lock LocaleMutex"), $name)
    };
    // Localize over the locale provided as first argument, taken by reference.
    ($locale:expr, $name:literal, $($key:literal -> $value:expr),+ $(,)*) => {{
        #[allow(clippy::fallible_impl_from)]
        {
            let mut errors = std::vec::Vec::new();
            let mut args = std::collections::HashMap::new();
            $(
                args.insert($key, fluent::FluentValue::from($value));
            )*
            let guard = &$locale;
            let msg = guard.bundle.get_message($name).expect("Missing translation name");
            let pattern = msg.value.expect("Message has no pattern");
            let formatted: std::string::String = guard.bundle.format_pattern(&pattern, Some(&args), &mut errors).to_string();
            assert_eq!(errors, std::vec::Vec::new());
            formatted
        }
    }};
    ($locale:expr, $name:literal) => {{
        #[allow(clippy::fallible_impl_from)]
        {
            let mut errors = std::vec::Vec::new();
            let guard = &$locale;
            let msg = guard.bundle.get_message($name).expect("Missing translation name");
            let pattern = msg.value.expect("Message has no pattern");
            let formatted: std::string::String = guard.bundle.format_pattern(&pattern, None, &mut errors).to_string();
            assert_eq!(errors, std::vec::Vec::new());
            formatted
        }
    }};
}

#[cfg(test)]
mod test {
    use crate::l10n::{load_locale_bundle, BVELanguage, BVELocale};

    macro_rules! loc_test {
        ($($tokens:tt)*) => {{
            let result = localize!($($tokens)*);
            assert!(!result.is_empty());
        }};
    }

    macro_rules! language_test {
        ($name:ident, $lang:ident) => {
            #[test]
            fn $name() {
                let language = load_locale_bundle(BVELocale::from_language(BVELanguage::$lang));
                loc_test!(language, "program-name");
                loc_test!(language, "language-code");
                loc_test!(language, "welcome", "name" -> "MyUsername");
            }
        };
    }

    language_test!(en, EN);
    language_test!(de, DE);
}
