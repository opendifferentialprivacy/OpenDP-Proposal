use std::any::{Any, TypeId};
use std::ffi::c_void;

use crate::{constructors, Transformation};
use crate::base::domain::{Domain, VectorDomain};

// #[no_mangle]
// pub extern "C" fn make_default_domain() -> *mut Domain {
//     Box::into_raw(Box::new(Domain::Vector(VectorDomain {
//         atomic_type: Box::new(Domain::numeric_scalar(None, None, false).unwrap()),
//         is_nonempty: false,
//         length: None
//     })))
// }
//
// #[no_mangle]
// pub extern "C" fn clamp_f64(
//     input_domain_ptr: *mut Domain, lower: f64, upper: f64
// ) -> *mut Transformation {
//     let input_domain = unsafe {
//         assert!(!input_domain_ptr.is_null());
//         &mut *input_domain_ptr
//     }.clone();
//
//     println!("input domain: {:?}", input_domain);
//
//     let clamper = constructors::preprocess::make_clamp_numeric(
//         input_domain,
//         lower.into(),
//         upper.into()).unwrap();
//
//     Box::into_raw(Box::new(clamper))
// }




// #[no_mangle]
// pub extern "C" fn build_f64(value: *mut f64) -> *mut c_void {
//     let any_value = value as *mut dyn Any;
//     let boxed_any_value = Box::new(any_value);
//     Box::into_raw(boxed_any_value) as *mut c_void
//     // let Box::new(value as *mut dyn Any);
//     // Box::new(unsafe {*value}) as Box<dyn Any>
// }
//
// #[no_mangle]
// pub extern "C" fn load_any(value: *mut c_void) {
//     let value: Box<Box<dyn Any>> = unsafe { Box::from_raw(value as *mut _) };
//     // match (**value).type_id() {
//     //     v if v == TypeId::of::<f64>() => {
//     //         println!("float {:?}", unsafe {*value as Box<dyn Any>}.downcast_ref::<f64>());
//     //     },
//     //     _ => panic!("unsupported type")
//     // }
// }

// #[no_mangle]
// pub extern fn load_f64_vec(samples_buf: *mut f64, length: u32) {
//     let data = unsafe {
//         std::slice::from_raw_parts_mut(samples_buf, length as usize)
//     }.to_vec();
//
//     println!("data {:?}", data);
// }


#[repr(u32)]
#[derive(Debug)]
pub enum Tag {
    F64,
    I64
}

#[repr(C)]
pub union Data {
    F64: f64,
    I64: i64,
}

#[repr(C)]
pub struct Value { tag: Tag, data: Data }

#[no_mangle]
pub extern "C" fn load_scalar(value: *mut Value) {
    // println!("value: {:?}", value);
    unsafe {
        match value.as_mut().expect("null pointer!") {
            Value { tag: Tag::F64, data: Data { F64: value } } => println!("F64: {:?}", value),
            Value { tag: Tag::I64, data: Data { I64: value } } => println!("I64: {:?}", value),
            _ => println!("Nothing"),
        }
    }
}
