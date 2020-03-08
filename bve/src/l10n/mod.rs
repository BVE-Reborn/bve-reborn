use crate::data::DATA;
use fluent::{FluentBundle, FluentResource, FluentValue};
use include_dir::File;
use locale_config::LanguageRange;
use nom::lib::std::collections::HashMap;
use once_cell::sync::Lazy;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::sync::RwLock;
use unic_langid::{langid, LanguageIdentifier};

pub static CURRENT_BVE_LOCALE: Lazy<RwLock<BVELocale>> = Lazy::new(|| RwLock::new(load_locale(get_current_language())));

pub struct BVELocale {
    language: Language,
    bundle: FluentBundle<FluentResource>,
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

#[must_use]
pub fn get_current_language() -> Language {
    let locale = locale_config::Locale::user_default();

    // Look for the message language
    if let Some((_category, range)) = locale.tags().find(|(c, _)| *c == Some("messages")) {
        parse_language_range(range)
    } else {
        // If it's not there, just grab the first one.
        if let Some((_category, range)) = locale.tags().next() {
            parse_language_range(range)
        } else {
            Language::EN
        }
    }
}

fn parse_language_range(range: LanguageRange<'_>) -> Language {
    let range_str = range.to_string();

    if range_str.is_empty() {
        return Language::EN;
    }

    let locale = unic_locale::Locale::from_bytes(range_str.as_bytes()).expect("Unable to parse locale");

    match locale.langid.language() {
        "de" => Language::DE,
        "en" | _ => Language::EN,
    }
}

#[test]
fn t() {
    let l = CURRENT_BVE_LOCALE.read().expect("Unable to lock LocaleMutex");
    let msg = l.bundle.get_message("welcome").expect("Missing translation name");
    let pattern = msg.value.expect("Message has no pattern");
    let mut errors = vec![];
    let mut args = HashMap::new();
    args.insert("name", FluentValue::from("Connor"));
    let value = l.bundle.format_pattern(&pattern, Some(&args), &mut errors);
    println!("{}", value);
}

#[must_use]
pub fn load_locale(language: Language) -> BVELocale {
    let mut bundle = FluentBundle::new(&[language.get_identifier(), Language::EN.get_identifier()]);

    // First load english as a baseline
    for x in load_resources(Language::EN) {
        bundle.add_resource(x).expect("Failed to add FTL resources to bundle")
    }

    // Then load other language
    if language != Language::EN {
        for x in load_resources(language) {
            bundle.add_resource_overriding(x);
        }
    }

    BVELocale { language, bundle }
}

#[must_use]
fn load_resources(language: Language) -> impl Iterator<Item = FluentResource> {
    enumerate_language_files(language).iter().map(|file| {
        FluentResource::try_new(String::from(
            file.contents_utf8()
                .unwrap_or_else(|| panic!("Translation file {} is not utf-8", file.path().display())),
        ))
        .unwrap_or_else(|_| panic!("Translation file {} is not valid ftl", file.path().display()))
    })
}

#[must_use]
pub fn enumerate_language_files(language: Language) -> &'static [File<'static>] {
    DATA.get_dir(format!("l10n/{}", language))
        .unwrap_or_else(|| panic!("Missing language dir {}", language))
        .files()
}
