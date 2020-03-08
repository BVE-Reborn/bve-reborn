use crate::data::DATA;
use crate::l10n::{BVELocale, Language};
use fluent::{FluentBundle, FluentResource};
use include_dir::File;

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
