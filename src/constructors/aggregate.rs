// use crate::base::{Domain, VectorDomain, ScalarDomain, Data};
// use crate::Transformation;


use crate::base::domain::{Domain, VectorDomain, ScalarDomain};
use crate::base::metric::DataDistance;
use crate::base::Data;

// fn make_sum(input_domain: Domain) -> Result<Transformation, Error> {
//
//     let output_domain = match &input_domain {
//         Domain::Vector(VectorDomain {
//                            atomic_type,
//                            is_nonempty,
//                            length
//                        }) => {
//
//             let atomic_type: &ScalarDomain = atomic_type.scalar()?;
//
//             let lower = Some(match (&atomic_type.nature.numeric()?.0, length) {
//                 (Some(lower), Some(length)) => lower * lower.cast(length),
//                 _ => None
//             });
//
//             let upper = Some(match (&atomic_type.nature.numeric()?.1, length) {
//                 (Some(upper), Some(length)) => upper * length,
//                 _ => None
//             });
//             Domain::numeric_scalar(lower, upper, atomic_type.may_have_nullity)?
//         },
//         _ => return Err(crate::Error::Raw("invalid input domain"))
//     };
//
//     Ok(Transformation {
//         input_domain,
//         output_domain,
//         stability_relation: Box::new(move |in_dist: &DataDistance, out_dist: &DataDistance| in_dist <= out_dist),
//         function: Box::new(move |data: Data| Ok(data)),
//         hint: None
//     })
// }