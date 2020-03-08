pub use current::*;
use fluent::{FluentBundle, FluentResource};
pub use load::*;
use once_cell::sync::Lazy;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::sync::RwLock;
use unic_langid::{langid, LanguageIdentifier};

mod current;
mod load;

pub static CURRENT_BVE_LOCALE: Lazy<RwLock<BVELocale>> = Lazy::new(|| RwLock::new(load_locale(get_current_language())));
pub static ENGLISH_LOCALE: Lazy<BVELocale> = Lazy::new(|| load_locale(Language::EN));

pub struct BVELocale {
    pub language: Language,
    pub bundle: FluentBundle<FluentResource>,
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
            Self::EN => write!(f, "en"),
            Self::DE => write!(f, "de"),
        }
    }
}

#[macro_export]
macro_rules! localize {
    // Localize in english for logging
    (@en $name:literal, $($key:literal -> $value:literal),+ $(,)*) => {
        $crate::localize!($crate::l10n::ENGLISH_LOCALE, $name, $($key -> $value),+)
    };
    (@en $name:literal) => {
        $crate::localize!($crate::l10n::ENGLISH_LOCALE, $name)
    };
    // Localize in the current locale
    ($name:literal, $($key:literal -> $value:literal),+ $(,)*) => {
        $crate::localize!($crate::l10n::CURRENT_BVE_LOCALE.read().expect("Unable to lock LocaleMutex"), $name, $($key -> $value),+)
    };
    ($name:literal) => {
        $crate::localize!($crate::l10n::CURRENT_BVE_LOCALE.read().expect("Unable to lock LocaleMutex"), $name)
    };
    // Localize over the locale provided as first argument, taken by reference.
    ($locale:expr, $name:literal, $($key:literal -> $value:literal),+ $(,)*) => {{
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
    }};
    ($locale:expr, $name:literal) => {{
        let mut errors = std::vec::Vec::new();
        let guard = &$locale;
        let msg = guard.bundle.get_message($name).expect("Missing translation name");
        let pattern = msg.value.expect("Message has no pattern");
        let formatted: std::string::String = guard.bundle.format_pattern(&pattern, None, &mut errors).to_string();
        assert_eq!(errors, std::vec::Vec::new());
        formatted
    }};
}

#[cfg(test)]
mod test {
    use crate::l10n::{load_locale, Language};

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
                let language = load_locale(Language::$lang);
                loc_test!(language, "name");
                loc_test!(language, "language-code");
                loc_test!(language, "welcome", "name" -> "MyUsername");
            }
        };
    }

    language_test!(en, EN);
    language_test!(de, DE);
}
