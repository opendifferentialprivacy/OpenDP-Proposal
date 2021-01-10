use std::marker::PhantomData;


// FRAMEWORK TYPES

pub trait Domain {
    type Carrier;
}

pub struct AllDomain<T> {
    _marker: PhantomData<T>
}
impl<T> AllDomain<T> {
    pub fn new() -> AllDomain<T> { AllDomain { _marker: PhantomData } }
}
impl<T> Domain for AllDomain<T> {
    type Carrier = T;
}

pub struct Operation<I, O> {
    pub in_domain: Box<dyn Domain<Carrier=I>>,
    pub out_domain: Box<dyn Domain<Carrier=O>>,
    function: Box<dyn Fn(*const I) -> Box<O>>,
}
impl<I, O> Operation<I, O> {
    pub fn new_all(function: impl Fn(&I) -> O + 'static) -> Operation<I, O> where I: 'static, O: 'static {
        Operation::new(Box::new(AllDomain::new()), Box::new(AllDomain::new()), function)
    }
    pub fn new(in_dom: Box<dyn Domain<Carrier=I>>, out_dom: Box<dyn Domain<Carrier=O>>, function: impl Fn(&I) -> O + 'static) -> Operation<I, O> {
        let function = move |arg: *const I| -> Box<O> {
            let arg = ffi_utils::as_ref(arg);
            let res = function(arg);
            Box::new(res)
        };
        let function = Box::new(function);
        Operation { in_domain: in_dom, out_domain: out_dom, function: function }
    }
    pub fn invoke(&self, arg: &I) -> O {
        let arg = arg as *const I;
        let res = (self.function)(arg);
        *res
    }
    pub fn invoke_ffi(&self, arg: &Box<I>) -> Box<O> {
        let arg = arg.as_ref() as *const I;
        (self.function)(arg)
    }
}



// CONSTRUCTORS

pub fn make_chain<I: 'static, X: 'static, O: 'static>(op1: Operation<X, O>, op0: Operation<I, X>) -> Operation<I, O> {
    let in_domain = op0.in_domain;
    let out_domain = op1.out_domain;
    let function1 = op1.function;
    let function0 = op0.function;
    let function = move |arg: *const I| -> Box<O> {
        let res0 = function0(arg);
        function1(&*res0)
    };
    let function = Box::new(function);
    Operation { in_domain, out_domain, function }
}

pub fn make_composition<I: 'static + Clone, O0: 'static, O1: 'static>(op0: Operation<I, O0>, op1: Operation<I, O1>) -> Operation<I, (Box<O0>, Box<O1>)> {
    let in_domain = op0.in_domain;
    let out_domain = Box::new(AllDomain { _marker: PhantomData });
    let function0 = op0.function;
    let function1 = op1.function;
    let function = move |arg: *const I| -> Box<(Box<O0>, Box<O1>)> {
        let res0 = function0(arg);
        let res1 = function1(arg);
        Box::new((res0, res1))
    };
    let function = Box::new(function);
    Operation { in_domain, out_domain, function }
}



// MONO & FFI_UTILS MODULES, COPIED HERE TO BE SELF-CONTAINED

pub mod mono {
    use std::any::{type_name, TypeId};
    use std::convert::{TryFrom, TryInto};
    use std::fmt::Debug;
    use std::mem::{size_of, align_of};
    use std::collections::HashMap;

    #[derive(Debug)]
    pub struct TypeError;

    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    pub struct Type {
        pub descriptor: String,
        pub size: usize,
        pub align: usize,
    }

    macro_rules! types {
        ($($type:ty),*) => { vec![$(Type::new::<$type>()),*] }
    }
    lazy_static! {
        static ref DESCRIPTOR_TO_TYPE: HashMap<String, Type> = {
            types![
                bool, char, u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, String
            ].into_iter().map(|e| (e.descriptor.clone(), e)).collect()
        };
    }

    impl Type {
        pub fn new<T: 'static>() -> Type {
            // TODO: Better generation of type descriptors.
            // Special case String, otherwise we get "alloc::string::String".
            let descriptor = if TypeId::of::<T>() == TypeId::of::<String>() {
                "String"
            } else {
                type_name::<T>()
            };
            let descriptor = descriptor.to_owned();
            let size = size_of::<T>();
            let align = align_of::<T>();
            Type { descriptor, size, align }
        }
        // Hacky special entry point for composition.
        pub fn new_box_pair(type0: &Type, type1: &Type) -> Type {
            let descriptor = format!("(Box<{}>, Box<{}>)", type0.descriptor, type1.descriptor);
            let size = 0;
            let align = 0;
            Type { descriptor, size, align }
        }
    }

    impl TryFrom<&str> for Type {
        type Error = TypeError;
        fn try_from(value: &str) -> Result<Self, Self::Error> {
            DESCRIPTOR_TO_TYPE.get(value).ok_or(TypeError).map(|e| e.clone())
        }
    }

    #[derive(Debug, Eq, PartialEq, Hash)]
    pub struct TypeArgs(pub Vec<Type>);

    impl TypeArgs {
        pub fn new(args: Vec<Type>) -> TypeArgs {
            TypeArgs(args)
        }
        pub fn descriptor(&self) -> String {
            let arg_descriptors: Vec<_> = self.0.iter().map(|e| e.descriptor.clone()).collect();
            format!("<{}>", arg_descriptors.join(", "))
        }
    }

    impl TryFrom<&str> for TypeArgs {
        type Error = TypeError;
        fn try_from(value: &str) -> Result<Self, Self::Error> {
            // TODO: Better TypeArgs parsing.
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
}

pub mod ffi_utils {

    pub fn into_raw<T>(o: T) -> *mut T {
        Box::into_raw(Box::<T>::new(o))
    }

    pub fn into_box<T, U>(o: T) -> Box<U> {
        let p = into_raw(o) as *mut U;
        unsafe { Box::from_raw(p) }
    }

    pub fn into_owned<T>(p: *mut T) -> T {
        assert!(!p.is_null());
        *unsafe { Box::<T>::from_raw(p) }
    }

    pub fn as_ref<'a, T>(p: *const T) -> &'a T {
        assert!(!p.is_null());
        unsafe { &*p }
    }

}



// FFI DEMO

pub mod ffi {
    use super::*;
    use super::mono::Type;

    pub struct FfiObject {
        pub type_: Type,
        pub value: Box<()>,
    }
    impl FfiObject {
        pub fn new_typed<T>(type_: Type, value: T) -> FfiObject {
            let value = ffi_utils::into_box(value);
            FfiObject { type_, value }
        }
        pub fn new<T: 'static>(value: T) -> FfiObject {
            let type_ = Type::new::<T>();
            Self::new_typed(type_, value)
        }
        pub fn into_owned<T>(self) -> T {
            let value = Box::into_raw(self.value) as *mut T;
            ffi_utils::into_owned(value)
        }
    }

    pub struct FfiOperation {
        pub input_type: Type,
        pub output_type: Type,
        pub value: Box<Operation<(), ()>>,
    }
    impl FfiOperation {
        pub fn new_typed<I, O>(input_type: Type, output_type: Type, value: Operation<I, O>) -> FfiOperation {
            let value = ffi_utils::into_box(value);
            FfiOperation { input_type, output_type, value }
        }
        pub fn new<I: 'static, O: 'static>(value: Operation<I, O>) -> FfiOperation {
            let input_type = Type::new::<I>();
            let output_type = Type::new::<O>();
            Self::new_typed(input_type, output_type, value)
        }
    }

    #[no_mangle]
    pub extern "C" fn invoke(operation: *const FfiOperation, arg: *mut FfiObject) -> *mut FfiObject {
        let operation = ffi_utils::as_ref(operation);
        let arg = ffi_utils::into_owned(arg);
        assert_eq!(arg.type_, operation.input_type);
        let res_type = operation.output_type.clone();
        let res = operation.value.invoke_ffi(&arg.value);
        let res = FfiObject::new_typed(res_type, res);
        ffi_utils::into_raw(res)
    }

    #[no_mangle]
    pub extern "C" fn make_chain(operation1: *mut FfiOperation, operation0: *mut FfiOperation) -> *mut FfiOperation {
        let operation0 = ffi_utils::into_owned(operation0);
        let operation1 = ffi_utils::into_owned(operation1);
        assert_eq!(operation0.output_type, operation1.input_type);
        let input_type = operation0.input_type.clone();
        let output_type = operation1.output_type.clone();
        let operation = super::make_chain(*operation1.value, *operation0.value);
        let operation = FfiOperation::new_typed(input_type, output_type, operation);
        ffi_utils::into_raw(operation)
    }

    #[no_mangle]
    pub extern "C" fn make_composition(operation0: *mut FfiOperation, operation1: *mut FfiOperation) -> *mut FfiOperation {
        let operation0 = ffi_utils::into_owned(operation0);
        let operation1 = ffi_utils::into_owned(operation1);
        assert_eq!(operation0.input_type, operation1.input_type);
        let input_type = operation0.input_type.clone();
        let output_type = Type::new_box_pair(&operation0.output_type, &operation1.output_type);
        let operation = super::make_composition(*operation0.value, *operation1.value);
        let operation = FfiOperation::new_typed(input_type, output_type, operation);
        ffi_utils::into_raw(operation)
    }

}



// UNIT TESTS

#[cfg(test)]
mod tests {
    use super::*;
    use super::ffi::{FfiObject, FfiOperation};

    fn make_ffi_operation<I: 'static, O: 'static>(func: impl Fn(&I) -> O + 'static) -> *mut FfiOperation {
        let op = Operation::new_all(func);
        let op = FfiOperation::new(op);
        ffi_utils::into_raw(op)
    }

    fn to_ffi<T: 'static>(obj: T) -> *mut FfiObject {
        let obj = FfiObject::new(obj);
        ffi_utils::into_raw(obj)
    }

    fn from_ffi<T>(obj: *mut FfiObject) -> T {
        let obj = ffi_utils::into_owned(obj);
        obj.into_owned()
    }

    #[test]
    fn test_invoke() {
        let op = make_ffi_operation(|a: &f32| a.clone());
        let a = 99.0_f32;
        let a = to_ffi(a);
        let r = ffi::invoke(op, a);
        let r: f32 = from_ffi(r);
        assert_eq!(r, 99.0);
    }

    #[test]
    fn test_make_chain() {
        let op0 = make_ffi_operation(|a: &u8| (a + 1) as i32);
        let op1 = make_ffi_operation(|a: &i32| (a + 1) as f64);
        let op = ffi::make_chain(op1, op0);
        let a = 99_u8;
        let a = to_ffi(a);
        let r = ffi::invoke(op, a);
        let r: f64 = from_ffi(r);
        assert_eq!(r, 101.0);
    }

    #[test]
    fn test_make_composition() {
        let op0 = make_ffi_operation(|a: &i32| (a + 1) as f32);
        let op1 = make_ffi_operation(|a: &i32| (a + 1) as f64);
        let op = ffi::make_composition(op0, op1);
        let a = 99;
        let a = to_ffi(a);
        let r = ffi::invoke(op, a);
        let r: (Box<f32>, Box<f64>) = from_ffi(r);
        assert_eq!(r, (Box::new(100.0_f32), Box::new(100.0_f64)));
    }

    #[test]
    fn test_make_composition_xxx() {
        let op0 = Operation::new_all(|a: &i32| (a + 1) as f32);
        let op1 = Operation::new_all(|a: &i32| (a + 1) as f64);
        let op = make_composition(op0, op1);
        let op = FfiOperation::new(op);
        let op = ffi_utils::into_raw(op);
        let a = 99;
        let a = to_ffi(a);
        let r = ffi::invoke(op, a);
        let r: (Box<f32>, Box<f64>) = from_ffi(r);
        assert_eq!(r, (Box::new(100.0_f32), Box::new(100.0_f64)));
    }

}
