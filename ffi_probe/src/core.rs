use std::rc::Rc;

use crate::data::{Data, Form, TraitObject};
use crate::dom::PairDomain;

pub trait Domain: TraitObject {
    fn box_clone(&self) -> Box<dyn Domain>;
    fn check_compatible(&self, other: &dyn Domain) -> bool;
    fn check_valid(&self, val: &Data) -> bool;
}
/// A smaller trait for the type-specific Domain stuff. I haven't figured out a way to dispatch
/// directly to one of these in check_valid() (avoiding the wrapping), but keeping it as a separate
/// trait for now.
pub trait DomainImpl {
    type Carrier: 'static + Form;
    fn check_valid_impl(&self, val: &Self::Carrier) -> bool;
}

// TODO: Make Function be Fn(&Carrier) -> Carrier ???
// TODO: Use impl Trait for Function?
pub type Function = Box<dyn Fn(&Data) -> Data>;

pub struct Operation {
    pub input_domain: Box<dyn Domain>,
    pub output_domain: Box<dyn Domain>,
    function: Rc<dyn Fn(&Data) -> Data>,
}

impl Operation {
    pub fn new(input_domain: impl Domain + 'static, output_domain: impl Domain + 'static, function: impl Fn(&Data) -> Data + 'static) -> Operation {
        Operation { input_domain: Box::new(input_domain), output_domain: Box::new(output_domain), function: Rc::new(function) }
    }
    pub fn invoke(&self, arg: &Data) -> Data {
        (self.function)(arg)
    }
}

// It's annoying that the arguments are moves rather than borrows, but this is necessary because the functions
// need to be moved into the new closure. The only alternative I could work out was to have the arguments
// be references with 'static lifetime, but that seemed even worse.
pub fn make_chain(operation1: Operation, operation0: Operation) -> Operation {
    assert!(operation0.output_domain.check_compatible(operation1.input_domain.as_ref()));
    let input_domain = operation0.input_domain;
    let output_domain = operation1.output_domain;
    let function0 = operation0.function;
    let function1 = operation1.function;
    let function = Rc::new(move |arg: &Data| -> Data {
        function1(&function0(arg))
    });
    Operation { input_domain, output_domain, function }
}

pub fn make_composition(operation0: Operation, operation1: Operation) -> Operation {
    assert!(operation0.input_domain.check_compatible(operation1.input_domain.as_ref()));
    let input_domain = operation0.input_domain;
    let output_domain0 = operation0.output_domain;
    let output_domain1 = operation1.output_domain;
    let output_domain = Box::new(PairDomain::<Data>::new(output_domain0, output_domain1));
    let function0 = operation0.function;
    let function1 = operation1.function;
    let function = Rc::new(move |arg: &Data| -> Data {
        let ret0 = function0(arg);
        let ret1 = function1(arg);
        let ret = (ret0, ret1);
        Data::new(ret)
    });
    Operation { input_domain, output_domain, function }
}



mod ffi {
    use std::os::raw::c_char;

    use crate::ffi_utils;

    use super::*;

    #[no_mangle]
    pub extern "C" fn opendp_core__operation_input_domain_clone(this: *const Operation) -> *mut Box<dyn Domain> {
        let this = ffi_utils::as_ref(this);
        let ret = this.input_domain.box_clone();
        ffi_utils::into_raw(ret)
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__operation_output_domain_clone(this: *const Operation) -> *mut Box<dyn Domain> {
        let this = ffi_utils::as_ref(this);
        let ret = this.output_domain.box_clone();
        ffi_utils::into_raw(ret)
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__operation_invoke(this: *const Operation, arg: *mut Data) -> *mut Data {
        let this = ffi_utils::as_ref(this);
        let arg = ffi_utils::as_ref(arg);
        let ret = this.invoke(arg);
        ffi_utils::into_raw(ret)
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__operation_free(this: *mut Operation) {
        ffi_utils::into_owned(this);
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__make_chain(operation1: *mut Operation, operation0: *mut Operation) -> *mut Operation {
        let operation1 = ffi_utils::into_owned(operation1);
        let operation0 = ffi_utils::into_owned(operation0);
        let ret = make_chain(operation1, operation0);
        ffi_utils::into_raw(ret)
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__make_composition(operation0: *mut Operation, operation1: *mut Operation) -> *mut Operation {
        let operation0 = ffi_utils::into_owned(operation0);
        let operation1 = ffi_utils::into_owned(operation1);
        let ret = make_composition(operation0, operation1);
        ffi_utils::into_raw(ret)
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__bootstrap() -> *const c_char {
        let spec =
r#"{
    "functions": [
        { "name": "operation_input_domain_clone", "args": [ ["const void *", "this"] ], "ret": "void *" },
        { "name": "operation_output_domain_clone", "args": [ ["const void *", "this"] ], "ret": "void *" },
        { "name": "operation_invoke", "args": [ ["const void *", "this"], ["void *", "arg"] ], "ret": "void *" },
        { "name": "operation_free", "args": [ ["void *", "this"] ] },
        { "name": "make_chain", "args": [ ["void *", "operation1"], ["void *", "operation0"] ], "ret": "void *" },
        { "name": "make_composition", "args": [ ["void *", "operation0"], ["void *", "operation1"] ], "ret": "void *" }
    ]
}"#;
        ffi_utils::bootstrap(spec)
    }

}


#[cfg(test)]
mod tests {
    use crate::dom::AllDomain;

    use super::*;

    #[test]
    fn test_identity() {
        let input_domain = AllDomain::<i32>::new();
        let output_domain = AllDomain::<i32>::new();
        let identity = Operation::new(input_domain, output_domain, |arg| Data::new(arg.as_form::<i32>().clone()));
        let arg = Data::new(10);
        let ret = identity.invoke(&arg);
        let ret: i32 = ret.into_form();
        assert_eq!(ret, 10);
    }

    #[test]
    fn test_make_chain() {
        let domain = AllDomain::<Data>::new();
        let operation0 = Operation::new(domain.clone(), domain.clone(), |arg| Data::new(arg.as_form::<i32>() + 1));
        let operation1 = Operation::new(domain.clone(), domain.clone(), |arg| Data::new(arg.as_form::<i32>() - 1));
        let chain = make_chain(operation1, operation0);
        let arg = Data::new(10);
        let ret = chain.invoke(&arg);
        let ret: i32 = ret.into_form();
        assert_eq!(ret, ret);
    }

    #[test]
    fn test_make_composition() {
        let domain = AllDomain::<i32>::new();
        let operation0 = Operation::new(domain.clone(), domain.clone(), |arg| Data::new(arg.as_form::<i32>() + 1));
        let operation1 = Operation::new(domain.clone(), domain.clone(), |arg| Data::new(arg.as_form::<i32>() - 1));
        let chain = make_composition(operation0, operation1);
        let arg = Data::new(10);
        let ret = chain.invoke(&arg);
        let ret: (Data, Data) = ret.into_form();
        let ret: (i32, i32) = (ret.0.into_form(), ret.1.into_form());
        assert_eq!(ret, (11, 9));
    }

}
