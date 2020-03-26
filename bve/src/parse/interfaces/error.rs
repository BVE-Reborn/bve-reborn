use crate::l10n::ForceEnglish;

// Temporary
pub trait UserError {
    fn to_data(&self) -> UserErrorData {
        UserErrorData {
            category: self.category(),
            line: self.line(),
            description: self.description(ForceEnglish::Local),
            description_english: self.description(ForceEnglish::English),
        }
    }

    fn category(&self) -> UserErrorCategory;
    fn line(&self) -> u64;

    fn description(&self, en: ForceEnglish) -> String;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UserErrorData {
    pub category: UserErrorCategory,
    pub line: u64,
    pub description: String,
    pub description_english: String,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum UserErrorCategory {
    Warning,
    Error,
}

impl UserError for () {
    fn to_data(&self) -> UserErrorData {
        unreachable!("Types that have () as their error/warning type should return only empty vectors of them.");
    }

    fn category(&self) -> UserErrorCategory {
        unreachable!("Types that have () as their error/warning type should return only empty vectors of them.");
    }

    fn line(&self) -> u64 {
        unreachable!("Types that have () as their error/warning type should return only empty vectors of them.");
    }

    fn description(&self, _en: ForceEnglish) -> String {
        unreachable!("Types that have () as their error/warning type should return only empty vectors of them.");
    }
}
