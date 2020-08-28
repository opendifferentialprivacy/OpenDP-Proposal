use crate::base::{Domain, Data};
use crate::metric::DataDistance;
use crate::Transformation;

pub mod chain;
pub mod preprocess;
pub mod aggregate;


pub fn make_row_transform(
    input_domain: Domain,
    output_domain: Domain,
    function: Box<dyn Fn(Data) -> Result<Data, crate::Error>>,
    hint: Option<Box<dyn Fn(&DataDistance, &DataDistance) -> DataDistance>>,
) -> Transformation {
    Transformation {
        input_domain,
        output_domain,
        stability_relation: Box::new(move |_input_distance: &DataDistance, _output_distance: &DataDistance| -> bool { true }),
        function,
        hint,
    }
}

