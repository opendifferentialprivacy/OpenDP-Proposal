use crate::base::{Data};
use crate::{Transformation, Error};
use crate::base::domain::Domain;
use crate::base::metric::DataDistance;

pub mod chain;
pub mod preprocess;
pub mod aggregate;


pub fn make_row_transform(
    input_domain: Domain,
    output_domain: Domain,
    function: Box<dyn Fn(Data) -> Result<Data, Error>>,
) -> Transformation {
    Transformation {
        input_domain,
        output_domain,
        stability_relation: Box::new(move |_input_distance: &DataDistance, _output_distance: &DataDistance| { Ok(true) }),
        function,
    }
}

