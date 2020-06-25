use crate::{Transformation, Domain};
use crate::traits::{PartialMin, PartialMax};
use std::fmt::Debug;


/// Create a transformation struct representing a clamp.
pub fn make_clamp<T>(
    lower: T, upper: T
) -> Result<Transformation<T, T>, &'static str>
    where T: 'static + PartialOrd + PartialMax + PartialMin + Clone + Debug {
    if lower > upper {
        return Err("lower must not be greater than upper")
    }

    /// Clamp potentially null data between lower/upper.
    fn clamper<T: PartialOrd + PartialMax + PartialMin>(
        lower: T, upper: T, data: Vec<T>,
    ) -> Result<Vec<T>, &'static str> {
        data.into_iter()
            .map(|v| v.partial_min(&upper)?.partial_max(&lower))
            .collect::<Result<_, _>>()
    }

    Ok(Transformation {
        input_domain: Domain::<T>::Continuous { lower: None, upper: None },
        output_domain: Domain::<T>::Continuous { lower: Some(lower.clone()), upper: Some(upper.clone()) },
        stability: 1.0,
        function: Box::new(move |data: Vec<T>| clamper(lower.clone(), upper.clone(), data)),
    })
}