// SPDX-License-Identifier: AGPL-3.0-or-later

use core::ffi::CStr;

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_foundation::{
    NSError, NSMutableArray, NSMutableDictionary, NSNumber, NSObject, NSProcessInfo, NSString,
};

pub fn nsstring_from_cstr(s: &CStr) -> Retained<NSString> {
    match s.to_str() {
        Ok(v) => NSString::from_str(v),
        Err(_) => NSString::from_str(""),
    }
}

pub fn nsstring_from_str(s: &str) -> Retained<NSString> {
    NSString::from_str(s)
}

pub fn nsstring_to_string(s: Option<&NSString>) -> String {
    match s {
        Some(v) => v.to_string(),
        None => String::new(),
    }
}

pub fn ns_mutable_array_with_capacity(capacity: usize) -> Retained<NSMutableArray<NSObject>> {
    NSMutableArray::<NSObject>::arrayWithCapacity(capacity)
}

pub fn ns_mutable_dictionary_with_capacity() -> Retained<NSMutableDictionary<NSString, NSObject>> {
    NSMutableDictionary::<NSString, NSObject>::new()
}

pub fn dict_set_str_key(
    dict: &NSMutableDictionary<NSString, NSObject>,
    key: &CStr,
    value: &NSObject,
) {
    let key_ns = nsstring_from_cstr(key);
    let key_proto = ProtocolObject::from_ref(&*key_ns);
    unsafe { dict.setObject_forKey(value, key_proto) };
}

pub fn operating_system_version_string() -> String {
    let info = NSProcessInfo::processInfo();
    info.operatingSystemVersionString().to_string()
}

pub fn ns_error_localized_description(err: &NSError) -> String {
    err.localizedDescription().to_string()
}

pub fn number_with_unsigned_int(value: u32) -> Retained<NSNumber> {
    NSNumber::new_u32(value)
}

pub fn number_with_bool(value: bool) -> Retained<NSNumber> {
    NSNumber::new_bool(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nsstring_round_trip() {
        let s = nsstring_from_cstr(c"hello fluxer");
        assert_eq!("hello fluxer", nsstring_to_string(Some(&s)));
    }

    #[test]
    fn nsstring_to_string_handles_none() {
        assert_eq!(String::new(), nsstring_to_string(None));
    }

    #[test]
    fn mutable_array_initial_count_is_zero() {
        let arr = ns_mutable_array_with_capacity(4);
        assert_eq!(0, arr.count());
    }

    #[test]
    fn mutable_dictionary_initial_count_is_zero() {
        let d = ns_mutable_dictionary_with_capacity();
        assert_eq!(0, d.count());
    }
}
