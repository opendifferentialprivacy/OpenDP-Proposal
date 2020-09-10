
use crate::{constructors, Transformation};
use crate::base_x::{Domain, VectorDomain};

#[no_mangle]
pub extern "C" fn make_default_domain() -> *mut Domain {
    Box::into_raw(Box::new(Domain::Vector(VectorDomain {
        atomic_type: Box::new(Domain::numeric_scalar(None, None, false).unwrap()),
        is_nonempty: false,
        length: None
    })))
}

#[no_mangle]
pub extern "C" fn clamp_f64(
    input_domain_ptr: *mut Domain, lower: f64, upper: f64
) -> *mut Transformation {
    let input_domain = unsafe {
        assert!(!input_domain_ptr.is_null());
        &mut *input_domain_ptr
    }.clone();

    println!("input domain: {:?}", input_domain);

    let clamper = constructors::preprocess::make_clamp_numeric(
        input_domain,
        lower.into(),
        upper.into()).unwrap();

    Box::into_raw(Box::new(clamper))
}