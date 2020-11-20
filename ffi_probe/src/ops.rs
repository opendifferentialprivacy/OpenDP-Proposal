use crate::data::*;


pub fn split_lines(arg: Data) -> Data {
    let form= arg.into_form::<String>();
    let lines: Vec<_> = form.lines().map(|e| e.to_owned()).collect();
    Data::new(lines)
}

// pub fn split_fields(arg: Data, count: i32) -> Data {
//     let form= arg.into_form::<Vec<String>>();
//     let lines: Vec<_> = form.lines().map(|e| e.to_owned()).collect();
//     println!("{:?}", lines);
//     Data::new(lines)
// }


pub fn make_identity<T: 'static + Primitive + Clone>() -> Box<dyn Fn(Data) -> Data> {
    let function = |arg: Data| {
        let form = arg.into_form::<T>();
        Data::new(form)
    };
    Box::new(function)
}


mod ffi {
    use super::*;
    use crate::ffi_utils;
    use std::os::raw::c_char;

    #[no_mangle]
    pub extern "C" fn opendp_ops__split_lines(arg: *mut Data) -> *mut Data {
        let arg = ffi_utils::into_owned(arg);
        let ret = split_lines(arg);
        ffi_utils::into_raw(ret)
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__bootstrap() -> *const c_char {
        let spec =
r#"{
    "functions": [
        { "name": "split_lines", "args": [ ["void *", "s"] ], "ret": "void *" }
    ]
}"#;
        ffi_utils::bootstrap(spec)
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lines() {
        let form = "ant\nbat\ncat\n".to_owned();
        let arg = Data::new(form);
        let ret = split_lines(arg);
        let lines = ret.into_form::<Vec<String>>();
        assert_eq!(lines, vec!["ant", "bat", "cat"]);
    }

}
