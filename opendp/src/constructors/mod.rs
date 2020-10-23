use crate::base::{Data};
use crate::{Transformation, Error};
use crate::base::domain::Domain;
use crate::base::metric::{Metric, Distance};

pub mod chain;
pub mod preprocess;
pub mod aggregate;
pub mod mechanisms;


pub fn make_row_transform<DI: Domain, DO: Domain, MI, MO>(
    input_domain: DI,
    input_metric: Metric,
    output_domain: DO,
    output_metric: Metric,
    function: Box<dyn Fn(Data<DI::Member>) -> Result<Data<DO::Member>, Error>>,
) -> Transformation<DI, DO, MI, MO>
    where MI: PartialOrd + Clone, MO: PartialOrd + Clone {
    Transformation {
        input_domain,
        input_metric,
        output_domain,
        output_metric,
        stability_relation: Box::new(move |_input_distance: &Distance<MI>, _output_distance: &Distance<MO>| { Ok(true) }),
        function,
    }
}

