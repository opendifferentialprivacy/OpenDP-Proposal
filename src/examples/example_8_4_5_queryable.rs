// issue: state has unknown type
// issue: cannot have access to data from this context


use crate::example_8_4_1_data_domain_enum::DataDomain;
use crate::example_8_4_2_metrics::*;
use crate::example_8_4_4_enum_relations::*;

struct DataPointer {
    id: i64
}

// struct InteractiveMeasurement {
//     input_domain: DataDomain,
//     privacy_loss: PrivacyDistance,
//     // issue: this function has unknown types
//     function: Box<dyn Fn(Measurement) -> Queryable<T, U>>
// }
//
// struct Queryable<T, U> {
//     state: T,
//     eval: Box<dyn Fn(T) -> (T, U)>
// }
//
// fn adaptive_composition(input_domain: DataDomain, privacy_loss: PrivacyDistance) -> InteractiveMeasurement {
//     InteractiveMeasurement {
//         input_domain,
//         privacy_loss,
//         function: Box::new(move |meas: Measurement, (data, loss)| {
//             let initial_state = ();
//
//             Queryable {
//                 state: (),
//                 eval: Box::new(())
//             }
//         })
//     }
// }