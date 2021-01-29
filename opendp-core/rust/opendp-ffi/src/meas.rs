use std::os::raw::c_char;

use opendp::meas;
use opendp::meas::AddNoise;

use crate::core::FfiMeasurement;
use crate::util;
use crate::util::TypeArgs;

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_laplace(type_args: *const c_char, sigma: f64) -> *mut FfiMeasurement {
    fn monomorphize<T>(sigma: f64) -> *mut FfiMeasurement where
        T: 'static + Copy + PartialEq + AddNoise {
        let measurement = meas::make_base_laplace::<T>(sigma);
        FfiMeasurement::new_from_types(measurement)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    dispatch!(monomorphize, [(type_args.0[0], @numbers)], (sigma))
}

#[no_mangle]
pub extern "C" fn opendp_meas__bootstrap() -> *const c_char {
    let spec =
r#"{
"functions": [
    { "name": "make_base_laplace", "args": [ ["const char *", "selector"], ["double", "sigma"] ], "ret": "void *" }
]
}"#;
    util::bootstrap(spec)
}
