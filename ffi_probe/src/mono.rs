use std::any::{type_name, TypeId};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;

#[derive(Debug)]
pub struct TypeError;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Type(String);

impl Type {
    pub fn new<T: 'static>() -> Type {
        // TODO: Better generation of type descriptors.
        // Special case String, otherwise we get "alloc::string::String".
        let descriptor = if TypeId::of::<T>() == TypeId::of::<String>() {
            "String"
        } else {
            type_name::<T>()
        };
        Type(format!("{}", descriptor))
    }
    pub fn descriptor(&self) -> String {
        self.0.clone()
    }
}

impl TryFrom<&str> for Type {
    type Error = TypeError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // TODO: Type validation.
        Ok(Type(value.to_owned()))
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct TypeArgs(Vec<Type>);

impl TypeArgs {
    pub fn new(args: Vec<Type>) -> TypeArgs {
        TypeArgs(args)
    }
    pub fn descriptor(&self) -> String {
        let arg_descriptors: Vec<_> = self.0.iter().map(|e| e.descriptor()).collect();
        format!("<{}>", arg_descriptors.join(", "))
    }
}

impl TryFrom<&str> for TypeArgs {
    type Error = TypeError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.starts_with("<") && value.ends_with(">") {
            let value = &value[1..value.len()-1];
            let split = value.split(",");
            let types: Result<Vec<_>, _> = split.into_iter().map(|e| e.trim().try_into()).collect();
            Ok(TypeArgs(types?))
        } else {
            Err(TypeError)
        }
    }
}


pub struct Dispatcher<T> {
    table: HashMap<TypeArgs, T>,
}
impl<T: 'static> Dispatcher<T> {
    pub fn new() -> Self {
        Dispatcher { table: HashMap::new() }
    }
    pub fn insert(&mut self, selector: TypeArgs, target: T) {
        self.table.insert(selector, target);
    }
    pub fn get(&self, selector: &TypeArgs) -> Option<&T> {
        self.table.get(selector)
    }
}


macro_rules! type_args {
    ($($type:ty),+) => {
        crate::mono::TypeArgs::new(vec![$(crate::mono::Type::new::<$type>()),+])
    }
}

macro_rules! register {
    ($dispatcher:ident, $function:ident, <$($type:ty),+>) => {
        $dispatcher.insert(type_args!($($type),+), $function::<$($type),+>())
    }
}

macro_rules! register_multi {
    ($dispatcher:ident, $function:ident, [$($type:ty),+]) => {
        $(register!($dispatcher, $function, <$type>);)+
    }
}


#[cfg(test)]
mod tests {
    use std::ffi::c_void;

    use crate::ffi_utils;

    use super::*;


    #[test]
    fn test_type() {
        let parsed: Type = "i32".try_into().unwrap();
        let explicit = Type::new::<i32>();
        assert_eq!(parsed, explicit);
    }

    #[test]
    fn test_type_args() {
        let parsed: TypeArgs = "<i32, f32>".try_into().unwrap();
        let explicit = type_args![i32, f32];
        assert_eq!(parsed, explicit);
    }

    #[derive(Debug, Eq, PartialEq)]
    struct Foo<T>(T);
    #[derive(Debug, Eq, PartialEq)]
    struct Bar<T>(T);
    #[derive(Debug, Eq, PartialEq)]
    struct Baz(String);
    fn go<G0, G1, A0: Debug, A1: Debug>(arg0: &Foo<A0>, arg1: &Bar<A1>, arg2: i32, arg3: f32) -> Baz {
        let val = format!(
            "go<{}, {}, {}, {}>({:?}, {:?}, {}, {})",
            type_name::<G0>(), type_name::<G1>(), type_name::<A0>(), type_name::<A1>(),
            arg0, arg1, arg2, arg3
        );
        Baz(val)
    }

    #[test]
    fn test_dispatch() {
        fn go_ffi(selector: &str, a0: *const c_void, a1: *const c_void, a2: i32, a3: f32) -> *mut c_void {
            fn monomorphize<G0: Debug, G1: Debug, A0: Debug, A1: Debug>() -> Box<dyn Fn(*const c_void, *const c_void, i32, f32) -> *mut c_void> {
                Box::new(|a0, a1, a2, a3| {
                    let arg0: &Foo<A0> = ffi_utils::as_ref(a0.cast());
                    let arg1: &Bar<A1> = ffi_utils::as_ref(a1.cast());
                    let ret = go::<G0, G1, A0, A1>(arg0, arg1, a2, a3);
                    ffi_utils::into_raw(ret).cast()
                })
            }

            let mut dispatcher: Dispatcher<Box<_>> = Dispatcher::new();
            register!(dispatcher, monomorphize, <i32, f32, i64, f64>);

            let selector = selector.try_into().unwrap();
            dispatcher.get(&selector).unwrap()(a0, a1, a2, a3)
        }
        let selector = "<i32, f32, i64, f64>";
        let a0 = ffi_utils::into_raw(Foo(9_i64)).cast();
        let a1 = ffi_utils::into_raw(Bar(9.9_f64)).cast();
        let a2 = 9;
        let a3 = 9.9;
        let ret = go_ffi(selector, a0, a1, a2, a3);
        let ret = ffi_utils::into_owned(ret as *mut Baz);
        assert_eq!(ret, Baz("go<i32, f32, i64, f64>(Foo(9), Bar(9.9), 9, 9.9)".to_owned()));
    }

}
