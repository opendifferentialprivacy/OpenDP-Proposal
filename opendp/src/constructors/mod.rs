use crate::base::{Data};
use crate::{Transformation, Error};
use crate::base::domain::Domain;
use crate::base::metric::DataDistance;

pub mod chain;
pub mod preprocess;
pub mod aggregate;
pub mod mechanisms;


pub fn make_row_transform<T, U>(
    input_domain: Domain<T>,
    output_domain: Domain<T>,
    function: Box<dyn Fn(Data<T>) -> Result<Data<U>, Error>>,
) -> Transformation<T, U> {
    Transformation {
        input_domain,
        output_domain,
        stability_relation: Box::new(move |_input_distance: &DataDistance, _output_distance: &DataDistance| { Ok(true) }),
        function,
    }
}

