
use crate::{constructors, Transformation, Error};
use crate::base::{Domain, VectorDomain, ScalarDomain, NumericDomain};


#[no_mangle]
pub extern "C" fn clamp_f64(lower: f64, upper: f64) -> *mut Result<Transformation, Error> {
    let input_domain = Domain::Vector(VectorDomain {
        atomic_type: Box::new(Domain::Scalar(ScalarDomain::Numeric(NumericDomain {
            lower: None, upper: None
        }))),
        is_nonempty: false,
        length: None
    });
    Box::into_raw(Box::new(constructors::make_clamp(input_domain, lower.into(), upper.into())))
}