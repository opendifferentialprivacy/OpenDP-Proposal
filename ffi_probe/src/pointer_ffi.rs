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
    function: Box<dyn Fn(*const I) -> *mut O>,
}
impl<I, O> Operation<I, O> {
    pub fn new_all(function: impl Fn(&I) -> O + 'static) -> Operation<I, O> where I: 'static, O: 'static {
        Operation::new(Box::new(AllDomain::new()), Box::new(AllDomain::new()), function)
    }
    pub fn new(in_dom: Box<dyn Domain<Carrier=I>>, out_dom: Box<dyn Domain<Carrier=O>>, function: impl Fn(&I) -> O + 'static) -> Operation<I, O> {
        let function = move |arg: *const I| -> *mut O {
            let arg = ffi_utils::as_ref(arg);
            let res = function(arg);
            ffi_utils::into_raw(res)
        };
        let function = Box::new(function);
        Operation { in_domain: in_dom, out_domain: out_dom, function: function }
    }
    pub fn invoke(&self, arg: &I) -> O {
        let arg = arg as *const I;
        let res = (self.function)(arg);
        ffi_utils::into_owned(res)
    }
    pub fn invoke_ffi(&self, arg: *const I) -> *mut O {
        (self.function)(arg)
    }
}



// CONSTRUCTORS

pub fn make_chain<I: 'static, X: 'static, O: 'static>(op1: Operation<X, O>, op0: Operation<I, X>) -> Operation<I, O> {
    let in_domain = op0.in_domain;
    let out_domain = op1.out_domain;
    let function1 = op1.function;
    let function0 = op0.function;
    let function = move |arg: *const I| -> *mut O {
        let intermediate = function0(arg) as *const X;
        let res = function1(intermediate);
        // Deallocate intermediate.
        ffi_utils::into_owned(intermediate as *mut X);
        res
    };
    let function = Box::new(function);
    Operation { in_domain, out_domain, function }
}

pub fn make_composition<I: 'static + Clone, O0: 'static, O1: 'static>(op0: Operation<I, O0>, op1: Operation<I, O1>) -> Operation<I, (Box<O0>, Box<O1>)> {
    let in_domain = op0.in_domain;
    let out_domain = Box::new(AllDomain { _marker: PhantomData });
    let function0 = op0.function;
    let function1 = op1.function;
    let function = move |arg: *const I| -> *mut (Box<O0>, Box<O1>) {
        let res0 = function0(arg);
        let res1 = function1(arg);
        let res0 = unsafe { Box::from_raw(res0) };
        let res1 = unsafe { Box::from_raw(res1) };
        ffi_utils::into_raw((res0, res1))
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
    use std::ffi::c_void;

    lazy_static! {
        static ref OPERATION_TYPE: Type = {
            Type::new::<Operation<(), ()>>()
        };
    }

    pub struct FfiObject {
        pub type_: Type,
        pub pointer: *mut c_void,
    }
    impl FfiObject {
        pub fn new_raw(type_: Type, pointer: *mut c_void) -> FfiObject {
            FfiObject { type_, pointer }
        }
        pub fn new<T: 'static>(obj: T) -> FfiObject {
            let type_ = Type::new::<T>();
            let pointer = ffi_utils::into_raw(obj) as *mut c_void;
            FfiObject { type_, pointer }
        }
    }

    pub struct FfiOperation {
        pub input_type: Type,
        pub output_type: Type,
        pub pointer: *mut Operation<(), ()>,
    }
    impl FfiOperation {
        pub fn new_raw(input_type: Type, output_type: Type, pointer: *mut Operation<(), ()>) -> FfiOperation {
            FfiOperation { input_type, output_type, pointer }
        }
        pub fn new<I: 'static, O: 'static>(operation: Operation<I, O>) -> FfiOperation {
            let input_type = Type::new::<I>();
            let output_type = Type::new::<O>();
            let pointer = ffi_utils::into_raw(operation) as *mut Operation<(), ()>;
            FfiOperation { input_type, output_type, pointer }
        }
        pub fn into_owned(self) -> Operation<(), ()> {
            ffi_utils::into_owned(self.pointer as *mut Operation<(), ()>)
        }
        pub fn as_ref(&self) -> &Operation<(), ()> {
            ffi_utils::as_ref(self.pointer as *const Operation<(), ()>)
        }
    }

    #[no_mangle]
    pub extern "C" fn invoke(operation: *const FfiOperation, arg: *mut FfiObject) -> *mut FfiObject {
        let operation = ffi_utils::as_ref(operation);
        let arg = ffi_utils::into_owned(arg);
        assert_eq!(arg.type_, operation.input_type);
        let res_type = operation.output_type.clone();
        let operation = operation.as_ref();
        let arg = arg.pointer as *mut ();
        let res = operation.invoke_ffi(arg);
        let res = FfiObject::new_raw(res_type, res as *mut c_void);
        ffi_utils::into_raw(res)
    }

    #[no_mangle]
    pub extern "C" fn make_chain(operation1: *mut FfiOperation, operation0: *mut FfiOperation) -> *mut FfiOperation {
        let operation0 = ffi_utils::into_owned(operation0);
        let operation1 = ffi_utils::into_owned(operation1);
        assert_eq!(operation0.output_type, operation1.input_type);
        let input_type = operation0.input_type.clone();
        let output_type = operation1.output_type.clone();
        let operation0 = operation0.into_owned();
        let operation1 = operation1.into_owned();
        let operation = super::make_chain(operation1, operation0);
        let operation = ffi_utils::into_raw(operation);
        let operation = FfiOperation::new_raw(input_type, output_type, operation);
        ffi_utils::into_raw(operation)
    }

    #[no_mangle]
    pub extern "C" fn make_composition(operation0: *mut FfiOperation, operation1: *mut FfiOperation) -> *mut FfiOperation {
        let operation0 = ffi_utils::into_owned(operation0);
        let operation1 = ffi_utils::into_owned(operation1);
        assert_eq!(operation0.input_type, operation1.input_type);
        let input_type = operation0.input_type.clone();
        let output_type = Type::new_box_pair(&operation0.output_type, &operation1.output_type);
        let operation0 = operation0.into_owned();
        let operation1 = operation1.into_owned();
        let operation = super::make_composition(operation0, operation1);
        let operation = ffi_utils::into_raw(operation) as *mut Operation<(), ()>;
        let operation = FfiOperation::new_raw(input_type, output_type, operation);
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

    fn from_ffi<T: 'static>(obj: *mut FfiObject) -> T {
        let obj = ffi_utils::into_owned(obj);
        ffi_utils::into_owned(obj.pointer as *mut T)
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
