use crate::l10n::{BVELanguage, BVELocale};
use locale_config::LanguageRange;

#[must_use]
pub fn get_current_language() -> BVELocale {
    let locale = locale_config::Locale::user_default();

    // Look for the message language
    if let Some((_category, range)) = locale.tags().find(|(c, _)| *c == Some("messages")) {
        parse_language_range(&range)
    } else {
        // If it's not there, just grab the first one.
        if let Some((_category, range)) = locale.tags().next() {
            parse_language_range(&range)
        } else {
            BVELocale::from_language(BVELanguage::EN)
        }
    }
}

fn parse_language_range(range: &LanguageRange<'_>) -> BVELocale {
    let range_str = range.to_string();

    if range_str.is_empty() {
        return BVELocale::from_language(BVELanguage::EN);
    }

    let locale = unic_locale::Locale::from_bytes(range_str.as_bytes()).expect("Unable to parse locale");

    BVELocale::from_ident(locale.langid)
}
