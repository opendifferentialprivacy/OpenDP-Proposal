use crate::data::{Data, Form, TraitObject};

pub trait Domain: TraitObject {
    type Carrier: 'static + Form;
    fn box_clone(&self) -> Box<dyn Domain<Carrier=Self::Carrier>>;
    fn check_compatible(&self, other: &dyn Domain<Carrier=Self::Carrier>) -> bool;
    fn check_valid(&self, val: &Data) -> bool {
        let val: &Self::Carrier = val.as_form();
        self.check_valid_impl(val)
    }
    fn check_valid_impl(&self, val: &Self::Carrier) -> bool;
}

// TODO: Make Function be Fn(&Carrier) -> Carrier
// TODO: Use impl Trait for Function?
pub type Function = Box<dyn Fn(&Data) -> Data>;

pub struct Operation<I, O> {
    input_domain: Box<dyn Domain<Carrier=I>>,
    output_domain: Box<dyn Domain<Carrier=O>>,
    function: Function,
}

impl<I: 'static + Form + Clone, O: 'static + Form + Clone> Operation<I, O> {
    pub fn new_with_domains(input_domain: impl Domain<Carrier=I> + 'static, output_domain: impl Domain<Carrier=O> + 'static, function: Function) -> Operation<I, O> {
        Operation { input_domain: Box::new(input_domain), output_domain: Box::new(output_domain), function }
    }
    pub fn new(function: Function) -> Operation<I, O> {
        let input_domain = crate::dom::AllDomain::<I>::new();
        let output_domain = crate::dom::AllDomain::<O>::new();
        Operation { input_domain: Box::new(input_domain), output_domain: Box::new(output_domain), function }
    }
    pub fn invoke(&self, arg: &Data) -> Data {
        (self.function)(arg)
    }
}

pub fn make_chain<I: 'static + Form + Clone, X: 'static + Form + Clone, O: 'static + Form + Clone>(operation1: Operation<X, O>, operation0: Operation<I, X>) -> Operation<I, O> {
    assert!(operation0.output_domain.check_compatible(operation1.input_domain.as_ref()));
    let function = move |arg: &Data| -> Data {
        operation1.invoke(&operation0.invoke(arg))
    };
    Operation::new(Box::new(function))
}



// mod ffi {
//     use super::*;
//     use crate::ffi_utils;
//     use std::os::raw::c_char;
//
//     #[no_mangle]
//     pub extern "C" fn opendp_core__make_chain(operation1: *mut Operation, operation0: *mut Operation) -> *mut Operation {
//         let operation1 = ffi_utils::into_owned(operation1);
//         let operation0 = ffi_utils::into_owned(operation0);
//         let ret = make_chain(operation1, operation0);
//         ffi_utils::into_raw(ret)
//     }
//
//     #[no_mangle]
//     pub extern "C" fn opendp_core__operation_invoke(this: *const Operation, arg: *mut Data) -> *mut Data {
//         let this = ffi_utils::as_ref(this);
//         let arg = ffi_utils::as_ref(arg);
//         let ret = this.invoke(arg);
//         ffi_utils::into_raw(ret)
//     }
//
//     #[no_mangle]
//     pub extern "C" fn opendp_core__operation_free(this: *mut Operation) {
//         ffi_utils::into_owned(this);
//     }
//
//     #[no_mangle]
//     pub extern "C" fn opendp_core__bootstrap() -> *const c_char {
//         let spec =
// r#"{
//     "functions": [
//         { "name": "make_chain", "args": [ ["void *", "operation1"], ["void *", "operation0"] ], "ret": "void *" },
//         { "name": "operation_invoke", "args": [ ["const void *", "this"], ["void *", "arg"] ], "ret": "void *" },
//         { "name": "operation_free", "args": [ ["void *", "this"] ] }
//     ]
// }"#;
//         ffi_utils::bootstrap(spec)
//     }
//
// }


#[cfg(test)]
mod tests {
    use crate::dom::AllDomain;

    use super::*;

    #[test]
    fn test_identity() {
        let input_domain = AllDomain::<Data>::new();
        let output_domain = AllDomain::<Data>::new();
        let identity: Operation<Data, Data> = Operation::new_with_domains(input_domain, output_domain, Box::new(|arg| arg.clone()));
        let form_in = "eat my shorts1!".to_owned();
        let arg = Data::new(form_in.clone());
        let ret = identity.invoke(&arg);
        let form_out: &String = ret.as_form();
        assert_eq!(&form_in, form_out);
    }

}
