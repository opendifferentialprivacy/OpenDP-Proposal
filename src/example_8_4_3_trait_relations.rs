// issue: measurement and transformation structs may not be serialized

use crate::example_8_4_1_data_domain_enum::*;
use crate::example_8_4_2_metrics::*;

struct Measurement {
    input_domain: Vec<DataDomain>,
    input_metric: DataMetric,
    // output_domain: Vec<DataDomain>
    output_metric: PrivacyMetric,
    variant: Box<dyn Relation>
}
struct Transformation {
    input_domain: Vec<DataDomain>,
    input_metric: DataMetric,
    output_domain: Vec<DataDomain>,
    output_metric: DataMetric,
    variant: Box<dyn Relation>
}


trait Relation {
    fn constant() -> Option<i64> {None}
    fn relation(&self, d_in: DataDistance, d_out: DataDistance) -> bool {
        if let Some(stability) = Self::constant() {
            return d_in * stability <= d_out
        }
        false
    }
}

struct Clamp;

impl Relation for Clamp {
    fn constant() -> Option<i64> {Some(1)}
}

