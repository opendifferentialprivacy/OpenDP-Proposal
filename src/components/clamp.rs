use crate::{Transformation, Domain, Properties};
use crate::traits::{PartialMin, PartialMax};
use std::fmt::Debug;


/// Create a transformation struct representing a clamp.
pub fn make_clamp<T>(
    input_properties: Properties<T>, lower: T, upper: T,
) -> Result<Transformation<T, T>, &'static str>
    where T: 'static + PartialOrd + PartialMax + PartialMin + Clone + Debug {
    if lower > upper {
        return Err("lower must not be greater than upper");
    }

    let mut output_properties = input_properties.clone();
    output_properties.domain = Some(Domain::<T>::Continuous {
        lower: Some(lower.clone()),
        upper: Some(upper.clone()),
    });

    Ok(Transformation {
        input_properties,
        output_properties,
        stability: 1.0,
        function: Box::new(move |data: Vec<T>|
            data.into_iter()
                .map(|v| v.partial_min(&upper)?.partial_max(&lower))
                .collect()),
    })
}