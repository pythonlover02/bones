use std::ffi::{CStr, c_char};

pub(crate) fn cstr_to_str<'a>(p: *const c_char) -> &'a str {
    match p.is_null() {
        true => "",
        false => unsafe { CStr::from_ptr(p).to_str().unwrap_or("") },
    }
}
