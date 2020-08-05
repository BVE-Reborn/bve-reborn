use crate::{owned_ptr_to_string, string_to_owned_ptr};
use bve::parse::{UserErrorCategory, UserErrorData};
use std::os::raw::c_char;

#[repr(C)]
pub struct User_Error_Data {
    pub category: UserErrorCategory,
    pub line: u64,
    pub description: *mut c_char,
    pub description_english: *mut c_char,
}

impl From<UserErrorData> for User_Error_Data {
    fn from(d: UserErrorData) -> Self {
        Self {
            category: d.category,
            line: d.line.unwrap_or(0),
            description: string_to_owned_ptr(d.description),
            description_english: string_to_owned_ptr(d.description_english),
        }
    }
}

impl Into<UserErrorData> for User_Error_Data {
    fn into(self) -> UserErrorData {
        UserErrorData {
            category: self.category,
            line: Some(self.line),
            description: unsafe { owned_ptr_to_string(self.description) },
            description_english: unsafe { owned_ptr_to_string(self.description_english) },
        }
    }
}
