pub mod data;
pub mod ops;


pub(crate) mod ffi_utils {
    use std::ffi::CStr;
    use std::ffi::CString;
    use std::os::raw::c_char;

    pub fn into_raw<T>(obj: T) -> *mut T {
        Box::into_raw(Box::<T>::new(obj))
    }

    pub fn into_owned<T>(ptr: *mut T) -> T {
        assert!(!ptr.is_null());
        *unsafe { Box::<T>::from_raw(ptr) }
    }

    // pub fn as_ref<T>(ptr: *const T) -> &T {
    //     assert!(!ptr.is_null());
    //     unsafe { &*ptr }
    // }
    //
    // pub fn as_mut<T>(ptr: *mut T) -> &mut T {
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

    // pub fn to_str(p: *mut c_char) -> &str {
    //     assert!(!p.is_null());
    //     let s = unsafe { CString::from_raw(p) };
    //     s.to_str().expect("Bad C string")
    // }

    pub fn clone_string(p: *const c_char) -> String {
        assert!(!p.is_null());
        let s= unsafe { CStr::from_ptr(p) };
        s.to_owned().into_string().expect("Bad C string")
    }

    pub fn bootstrap(spec: &str) -> *const c_char {
        into_c_char_p(spec.to_owned())
    }

}
