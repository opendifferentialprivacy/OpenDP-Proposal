use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;


pub fn into_raw<T>(o: T) -> *mut T {
    Box::into_raw(Box::<T>::new(o))
}

pub fn into_box<T, U>(o: T) -> Box<U> {
    let p = into_raw(o) as *mut U;
    unsafe { Box::from_raw(p) }
}

// pub fn as_raw<T>(o: &T) -> *mut T {
//     o as *mut T
// }

pub fn into_owned<T>(p: *mut T) -> T {
    assert!(!p.is_null());
    *unsafe { Box::<T>::from_raw(p) }
}

pub fn as_ref<'a, T>(p: *const T) -> &'a T {
    assert!(!p.is_null());
    unsafe { &*p }
}

// pub fn as_mut<'a, T>(ptr: *mut T) -> &'a mut T {
//     assert!(!ptr.is_null());
//     unsafe { &mut *ptr }
// }

pub fn into_c_char_p(s: String) -> *mut c_char {
    CString::new(s).unwrap().into_raw()
}

// pub fn into_string(p: *mut c_char) -> String {
//     assert!(!p.is_null());
//     let s = unsafe { CString::from_raw(p) };
//     s.into_string().expect("Bad C string")
// }

pub fn to_str<'a>(p: *const c_char) -> &'a str {
    assert!(!p.is_null());
    let s = unsafe { CStr::from_ptr(p) };
    s.to_str().expect("Bad C string")
}

pub fn to_option_str<'a>(p: *const c_char) -> Option<&'a str> {
    if !p.is_null() {
        Some(to_str(p))
    } else {
        None
    }
}

pub fn bootstrap(spec: &str) -> *const c_char {
    // FIXME: Leaks string.
    into_c_char_p(spec.to_owned())
}

#[allow(non_camel_case_types)]
pub type c_bool = u8;  // PLATFORM DEPENDENT!!!

pub fn to_bool(b: c_bool) -> bool {
    if b != 0 { true } else { false }
}
