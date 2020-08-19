use opendp_proposal;
use opendp_proposal::base::{AtomicValue, Transformation};
use opendp_proposal::constructors::*;
use opendp_proposal::domain::{AtomicDomain, DataDomain, I64Domain, IntDomain, Scalar, Vector};
use opendp_proposal::metric::{AddRemove, DataMetric};

// generic perks:
// - any (one) atomic type

// enum perks:
// - recursive structures
// - non-homogeneous atomic types


// examples of bindings
// develop Value enum route
// secure ffi






// fn example_usage() -> Result<(), &'static str> {
//     let original_domain = DataDomain::Vector(Vector {
//         length: Some(100),
//         is_nonempty: false,
//         atomic_type: Box::new(DataDomain::Scalar(Scalar(AtomicDomain::Int(IntDomain::I64(I64Domain {
//             lower: Some(2),
//             upper: Some(10),
//             categories: None,
//         }))))),
//     });
//
//     let clamp = make_clamp_numeric(
//         original_domain.clone(),
//         DataMetric::AddRemove(AddRemove),
//         0., 10.)?;
//
//     // ILLEGAL
//     // clamp.output_domain = original_domain;
//
//     let sum = make_sum(
//         clamp.output_domain(),
//         DataMetric::AddRemove(AddRemove))?;
//
//     make_tt_chain(sum, clamp, None)?;
//
//     println!("{:?}", sum.input_domain());
//     Ok(())
// }



// fn example_usage_normalized_atomics() -> Result<(), &'static str> {
//
//     let original_domain = DataDomain::Vector(Vector {
//         length: Some(100),
//         is_nonempty: false,
//         atomic_type: Box::new(DataDomain::Scalar(Scalar(AtomicDomain::Int(IntDomain::I64(I64Domain {
//             lower: Some(2),
//             upper: Some(10),
//             categories: None
//         }))))),
//     });
//
//     let clamp = make_clamp_numeric(
//         original_domain,
//         DataMetric::AddRemove(AddRemove),
//         // inputs are
//         0.0.into(), 10.0.into())?;
//
//     let sum = make_sum(
//         clamp.output_domain(),
//         DataMetric::AddRemove(AddRemove))?;
//
//     make_tt_chain(sum, clamp, None)?;
//
//     println!("{:?}", sum.input_domain());
//     Ok(())
// }

fn main() {
    // example_usage().unwrap();
}
