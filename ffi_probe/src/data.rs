use std::any::Any;
use std::collections::HashMap;


pub trait Element {}
impl Element for i8 {}
impl Element for i16 {}
impl Element for i32 {}
impl Element for i64 {}
impl Element for u8 {}
impl Element for u16 {}
impl Element for u32 {}
impl Element for u64 {}
impl Element for f32 {}
impl Element for f64 {}
impl Element for bool {}
impl Element for String {}
impl Element for Data {}
pub trait Primitive: Element {}
impl Primitive for i8 {}
impl Primitive for i16 {}
impl Primitive for i32 {}
impl Primitive for i64 {}
impl Primitive for u8 {}
impl Primitive for u16 {}
impl Primitive for u32 {}
impl Primitive for u64 {}
impl Primitive for f32 {}
impl Primitive for f64 {}
impl Primitive for bool {}
impl Primitive for String {}

pub trait Form {
    fn as_any(&self) -> &dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
    fn box_clone(&self) -> Box<dyn Form>;
}

impl<T> Form for T where
    T: 'static + Primitive + Clone {
    // Not sure if it's better to use into_any() (which consumes the Form) or or as_any() (returns ref), trying both for now.
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
    fn box_clone(&self) -> Box<dyn Form> { Box::new(self.clone()) }
}

impl<T> Form for (T,) where
    T: 'static + Element + Clone {
    fn as_any(&self) -> &dyn Any { self }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn box_clone(&self) -> Box<dyn Form> { Box::new(self.clone()) }
}

impl<T0, T1> Form for (T0, T1) where
    T0: 'static + Element + Clone,
    T1: 'static + Element + Clone {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
    fn box_clone(&self) -> Box<dyn Form> { Box::new(self.clone()) }
}

impl<T0, T1, T2> Form for (T0, T1, T2) where
    T0: 'static + Element + Clone,
    T1: 'static + Element + Clone,
    T2: 'static + Element + Clone {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
    fn box_clone(&self) -> Box<dyn Form> { Box::new(self.clone()) }
}

impl<T> Form for HashMap<String, T> where
    T: 'static + Element + Clone {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
    fn box_clone(&self) -> Box<dyn Form> { Box::new(self.clone()) }
}

impl<T> Form for HashMap<usize, T> where
    T: 'static + Element + Clone {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
    fn box_clone(&self) -> Box<dyn Form> { Box::new(self.clone()) }
}

impl<T> Form for Vec<T> where
    T: 'static + Element + Clone {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
    fn box_clone(&self) -> Box<dyn Form> { Box::new(self.clone()) }
}


pub struct Data {
    form: Box<dyn Form>,
}

impl Data {
    pub fn new<T: 'static + Form>(form: T) -> Data {
        Data { form: Box::new(form) }
    }
    pub fn as_form<T: 'static + Form>(&self) -> &T {
        self.form.as_any().downcast_ref::<T>().expect("Wrong form")
    }
    pub fn into_form<T: 'static + Form>(self) -> T {
        let any= self.form.into_any();
        let result= any.downcast::<T>();
        *result.expect("Wrong form")
    }
}

impl Clone for Data {
    fn clone(&self) -> Self {
        Data { form: self.form.box_clone() }
    }
}



pub mod ffi {
    use super::*;
    use crate::ffi_utils;
    use std::os::raw::c_char;

    #[no_mangle]
    pub unsafe extern "C" fn opendp_data__new_string(p: *const c_char) -> *mut Data {
        let s = ffi_utils::clone_string(p);
        let data = Data::new(s);
        ffi_utils::into_raw(data)
    }

    #[no_mangle]
    pub extern "C" fn opendp_data__bootstrap() -> *const c_char {
        let spec =
r#"{
    "functions": [
        { "name": "new_string", "args": [ ["const char *", "s"] ], "ret": "void *" }
    ]
}"#;
        ffi_utils::bootstrap(spec)
    }

}



#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fmt::Debug;

    #[test]
    fn test_scalar() {
        let form = 99.9;
        test_round_trip(form);
    }

    #[test]
    fn test_tuple() {
        let form = (1, 2, 3);
        test_round_trip(form);
    }

    #[test]
    fn test_map() {
        let form: HashMap<_, _> = vec![("foo".to_string(), 1), ("bar".to_string(), 2)].into_iter().collect();
        test_round_trip(form);
    }

    #[test]
    fn test_vec() {
        let form = vec![1, 2, 3];
        test_round_trip(form);
    }

    #[test]
    fn test_nested() {
        let form = (Data::new(vec![1, 2, 3]), Data::new(99.9));
        test_sanity(form);
    }

    #[test]
    #[should_panic]
    fn test_bogus() {
        let form = (Data::new(vec![1, 2, 3]), Data::new(99.9));
        test_insanity(form);
    }

    fn test_round_trip<T: 'static + Form + Clone + PartialEq + Debug>(form: T) {
        let data = Data::new(form.clone());
        let retrieved: T = data.into_form();
        assert_eq!(form, retrieved);
    }

    fn test_sanity<T: 'static + Form>(form: T) {
        let data = Data::new(form);
        let _retrieved: T = data.into_form();
    }

    fn test_insanity<T: 'static + Form>(form: T) {
        let data = Data::new(form);
        let _retrieved: (f64, f64) = data.into_form();
    }

}
