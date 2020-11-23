use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;

pub trait Element: Debug {}
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

pub trait Form: Debug {
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
    fn as_any(&self) -> &dyn Any;
    fn box_clone(&self) -> Box<dyn Form>;
    fn eq(&self, other: &dyn Any) -> bool;
}

impl Form for Data where {
    // Not sure if it's better to use into_any() (which consumes the Form) or or as_any() (returns ref), trying both for now.
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
    fn box_clone(&self) -> Box<dyn Form> { Box::new(self.clone()) }
    fn eq(&self, other: &dyn Any) -> bool { other.downcast_ref::<Self>().map_or(false, |o| o == self) }
}

impl<T> Form for T where
    T: 'static + Primitive + Clone + PartialEq {
    // Not sure if it's better to use into_any() (which consumes the Form) or or as_any() (returns ref), trying both for now.
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
    fn box_clone(&self) -> Box<dyn Form> { Box::new(self.clone()) }
    fn eq(&self, other: &dyn Any) -> bool { other.downcast_ref::<Self>().map_or(false, |o| o == self) }
}

impl<T0> Form for (T0,) where
    T0: 'static + Element + Clone + PartialEq {
    fn as_any(&self) -> &dyn Any { self }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn box_clone(&self) -> Box<dyn Form> { Box::new(self.clone()) }
    fn eq(&self, other: &dyn Any) -> bool { other.downcast_ref::<Self>().map_or(false, |o| o == self) }
}

impl<T0, T1> Form for (T0, T1) where
    T0: 'static + Form + Clone + PartialEq,
    T1: 'static + Form + Clone + PartialEq {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
    fn box_clone(&self) -> Box<dyn Form> { Box::new(self.clone()) }
    fn eq(&self, other: &dyn Any) -> bool { other.downcast_ref::<Self>().map_or(false, |o| o == self) }
}

impl<T0, T1, T2> Form for (T0, T1, T2) where
    T0: 'static + Element + Clone + PartialEq,
    T1: 'static + Element + Clone + PartialEq,
    T2: 'static + Element + Clone + PartialEq {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
    fn box_clone(&self) -> Box<dyn Form> { Box::new(self.clone()) }
    fn eq(&self, other: &dyn Any) -> bool { other.downcast_ref::<Self>().map_or(false, |o| o == self) }
}

impl<T0, T1, T2, T3> Form for (T0, T1, T2, T3) where
    T0: 'static + Element + Clone + PartialEq,
    T1: 'static + Element + Clone + PartialEq,
    T2: 'static + Element + Clone + PartialEq,
    T3: 'static + Element + Clone + PartialEq {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
    fn box_clone(&self) -> Box<dyn Form> { Box::new(self.clone()) }
    fn eq(&self, other: &dyn Any) -> bool { other.downcast_ref::<Self>().map_or(false, |o| o == self) }
}

impl<T0, T1, T2, T3, T4> Form for (T0, T1, T2, T3, T4) where
    T0: 'static + Element + Clone + PartialEq,
    T1: 'static + Element + Clone + PartialEq,
    T2: 'static + Element + Clone + PartialEq,
    T3: 'static + Element + Clone + PartialEq,
    T4: 'static + Element + Clone + PartialEq {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
    fn box_clone(&self) -> Box<dyn Form> { Box::new(self.clone()) }
    fn eq(&self, other: &dyn Any) -> bool { other.downcast_ref::<Self>().map_or(false, |o| o == self) }
}

impl<T0, T1, T2, T3, T4, T5> Form for (T0, T1, T2, T3, T4, T5) where
    T0: 'static + Element + Clone + PartialEq,
    T1: 'static + Element + Clone + PartialEq,
    T2: 'static + Element + Clone + PartialEq,
    T3: 'static + Element + Clone + PartialEq,
    T4: 'static + Element + Clone + PartialEq,
    T5: 'static + Element + Clone + PartialEq {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
    fn box_clone(&self) -> Box<dyn Form> { Box::new(self.clone()) }
    fn eq(&self, other: &dyn Any) -> bool { other.downcast_ref::<Self>().map_or(false, |o| o == self) }
}

impl<T0, T1, T2, T3, T4, T5, T6> Form for (T0, T1, T2, T3, T4, T5, T6) where
    T0: 'static + Element + Clone + PartialEq,
    T1: 'static + Element + Clone + PartialEq,
    T2: 'static + Element + Clone + PartialEq,
    T3: 'static + Element + Clone + PartialEq,
    T4: 'static + Element + Clone + PartialEq,
    T5: 'static + Element + Clone + PartialEq,
    T6: 'static + Element + Clone + PartialEq {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
    fn box_clone(&self) -> Box<dyn Form> { Box::new(self.clone()) }
    fn eq(&self, other: &dyn Any) -> bool { other.downcast_ref::<Self>().map_or(false, |o| o == self) }
}

impl<T0, T1, T2, T3, T4, T5, T6, T7> Form for (T0, T1, T2, T3, T4, T5, T6, T7) where
    T0: 'static + Element + Clone + PartialEq,
    T1: 'static + Element + Clone + PartialEq,
    T2: 'static + Element + Clone + PartialEq,
    T3: 'static + Element + Clone + PartialEq,
    T4: 'static + Element + Clone + PartialEq,
    T5: 'static + Element + Clone + PartialEq,
    T6: 'static + Element + Clone + PartialEq,
    T7: 'static + Element + Clone + PartialEq {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
    fn box_clone(&self) -> Box<dyn Form> { Box::new(self.clone()) }
    fn eq(&self, other: &dyn Any) -> bool { other.downcast_ref::<Self>().map_or(false, |o| o == self) }
}

impl<T0, T1, T2, T3, T4, T5, T6, T7, T8> Form for (T0, T1, T2, T3, T4, T5, T6, T7, T8) where
    T0: 'static + Element + Clone + PartialEq,
    T1: 'static + Element + Clone + PartialEq,
    T2: 'static + Element + Clone + PartialEq,
    T3: 'static + Element + Clone + PartialEq,
    T4: 'static + Element + Clone + PartialEq,
    T5: 'static + Element + Clone + PartialEq,
    T6: 'static + Element + Clone + PartialEq,
    T7: 'static + Element + Clone + PartialEq,
    T8: 'static + Element + Clone + PartialEq {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
    fn box_clone(&self) -> Box<dyn Form> { Box::new(self.clone()) }
    fn eq(&self, other: &dyn Any) -> bool { other.downcast_ref::<Self>().map_or(false, |o| o == self) }
}

impl<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9> Form for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9) where
    T0: 'static + Element + Clone + PartialEq,
    T1: 'static + Element + Clone + PartialEq,
    T2: 'static + Element + Clone + PartialEq,
    T3: 'static + Element + Clone + PartialEq,
    T4: 'static + Element + Clone + PartialEq,
    T5: 'static + Element + Clone + PartialEq,
    T6: 'static + Element + Clone + PartialEq,
    T7: 'static + Element + Clone + PartialEq,
    T8: 'static + Element + Clone + PartialEq,
    T9: 'static + Element + Clone + PartialEq {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
    fn box_clone(&self) -> Box<dyn Form> { Box::new(self.clone()) }
    fn eq(&self, other: &dyn Any) -> bool { other.downcast_ref::<Self>().map_or(false, |o| o == self) }
}

impl<T> Form for HashMap<String, T> where
    T: 'static + Form + Clone + PartialEq {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
    fn box_clone(&self) -> Box<dyn Form> { Box::new(self.clone()) }
    fn eq(&self, other: &dyn Any) -> bool { other.downcast_ref::<Self>().map_or(false, |o| o == self) }
}

impl<T> From<HashMap<String, T>> for Data
    where T: 'static + Form + Clone + PartialEq {
    fn from(src: HashMap<String, T>) -> Self {
        Data::new(src)
    }
}

impl<T> Form for Vec<T> where
    T: 'static + Form + Clone + PartialEq {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
    fn box_clone(&self) -> Box<dyn Form> { Box::new(self.clone()) }
    fn eq(&self, other: &dyn Any) -> bool { other.downcast_ref::<Self>().map_or(false, |o| o == self) }
}

impl<T> From<Vec<T>> for Data
    where T: 'static + Form + Clone + PartialEq {
    fn from(src: Vec<T>) -> Self {
        Data::new(src)
    }
}


#[derive(Debug)]
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

impl PartialEq for Data {
    fn eq(&self, other: &Self) -> bool {
        self.form.eq(other.form.as_any())
    }
}


pub mod ffi {
    use std::os::raw::c_char;

    use crate::ffi_utils;

    use super::*;

    #[no_mangle]
    pub extern "C" fn opendp_data__from_string(p: *const c_char) -> *mut Data {
        let s = ffi_utils::to_str(p).to_owned();
        let data = Data::new(s);
        ffi_utils::into_raw(data)
    }

    #[no_mangle]
    pub extern "C" fn opendp_data__to_string(this: *const Data) -> *const c_char {
        let this = ffi_utils::as_ref(this);
        // FIXME: This probably isn't the right way to do this, but not sure how to get to_string() on Data.
        let string: String = format!("{:?}", this);
        // FIXME: Leaks string.
        ffi_utils::into_c_char_p(string)
    }

    #[no_mangle]
    pub extern "C" fn opendp_data__data_free(this: *mut Data) {
        ffi_utils::into_owned(this);
    }

    #[no_mangle]
    pub extern "C" fn opendp_data__bootstrap() -> *const c_char {
        let spec =
r#"{
    "functions": [
        { "name": "from_string", "args": [ ["const char *", "s"] ], "ret": "void *" },
        { "name": "to_string", "args": [ ["void *", "this"] ], "ret": "const char *" },
        { "name": "data_free", "args": [ ["void *", "this"] ] }
    ]
}"#;
        ffi_utils::bootstrap(spec)
    }

}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fmt::Debug;

    use super::*;

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
        let form: HashMap<_, _> = vec![("foo".to_owned(), 1), ("bar".to_owned(), 2)].into_iter().collect();
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
        test_round_trip(form);
    }

    #[test]
    #[should_panic]
    fn test_bogus() {
        let form = (Data::new(vec![1, 2, 3]), Data::new(99.9));
        let data = Data::new(form);
        let _retrieved: (f64, f64) = data.into_form();
    }

    fn test_round_trip<T: 'static + Form + Clone + PartialEq + Debug>(form: T) {
        let data = Data::new(form.clone());
        let retrieved: T = data.into_form();
        assert_eq!(form, retrieved);
    }
}
