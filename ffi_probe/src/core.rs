use crate::data::Data;


pub type Function = Box<dyn Fn(&Data) -> Data>;

pub struct Operation {
    function: Function,
}

impl Operation {
    pub fn new(function: Function) -> Operation {
        Operation { function }
    }
    pub fn invoke(&self, arg: &Data) -> Data {
        (self.function)(arg)
    }
}

pub fn make_chain(operation1: Operation, operation0: Operation) -> Operation {
    let function = move |arg: &Data| -> Data {
        operation1.invoke(&operation0.invoke(arg))
    };
    Operation::new(Box::new(function))
}


mod ffi {
    use super::*;
    use crate::ffi_utils;
    use std::os::raw::c_char;

    #[no_mangle]
    pub extern "C" fn opendp_core__make_chain(operation1: *mut Operation, operation0: *mut Operation) -> *mut Operation {
        let operation1 = ffi_utils::into_owned(operation1);
        let operation0 = ffi_utils::into_owned(operation0);
        let ret = make_chain(operation1, operation0);
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
    pub extern "C" fn opendp_core__bootstrap() -> *const c_char {
        let spec =
r#"{
    "functions": [
        { "name": "make_chain", "args": [ ["void *", "operation1"], ["void *", "operation0"] ], "ret": "void *" },
        { "name": "operation_invoke", "args": [ ["const void *", "this"], ["void *", "arg"] ], "ret": "void *" },
        { "name": "operation_free", "args": [ ["void *", "this"] ] }
    ]
}"#;
        ffi_utils::bootstrap(spec)
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let identity = Operation::new(Box::new(|arg| arg.clone()));
        let form_in = "eat my shorts1!".to_owned();
        let arg = Data::new(form_in.clone());
        let ret = identity.invoke(&arg);
        let form_out: &String = ret.as_form();
        assert_eq!(&form_in, form_out);
    }

}
