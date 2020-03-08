use crate::data::DATA;
use crate::l10n::{BVELanguage, BVELocale, BVELocaleBundle};
use fluent::{FluentBundle, FluentResource};
use include_dir::File;

#[must_use]
pub fn load_locale_bundle(language: BVELocale) -> BVELocaleBundle {
    let english_locale = BVELocale::from_language(BVELanguage::EN);
    let mut bundle = FluentBundle::new(&[language.langid.clone(), english_locale.langid.clone()]);

    // First load english as a baseline
    for x in load_resources(&english_locale) {
        bundle.add_resource(x).expect("Failed to add FTL resources to bundle")
    }

    // Then load other language
    if language.lang != BVELanguage::EN {
        for x in load_resources(&language) {
            bundle.add_resource_overriding(x);
        }
    }

    BVELocaleBundle { language, bundle }
}

fn load_resources(language: &BVELocale) -> impl Iterator<Item = FluentResource> {
    enumerate_language_files(language).iter().map(|file| {
        FluentResource::try_new(String::from(
            file.contents_utf8()
                .unwrap_or_else(|| panic!("Translation file {} is not utf-8", file.path().display())),
        ))
        .unwrap_or_else(|_| panic!("Translation file {} is not valid ftl", file.path().display()))
    })
}

#[must_use]
pub fn enumerate_language_files(language: &BVELocale) -> &'static [File<'static>] {
    DATA.get_dir(format!("l10n/{}", language.lang))
        .unwrap_or_else(|| panic!("Missing language dir {}", language.lang))
        .files()
}
