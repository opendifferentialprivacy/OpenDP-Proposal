use crate::{Transformation, Bounds, Domain};
use crate::traits::{PartialMin, PartialMax};
use std::fmt::Debug;

pub trait IsClampable {}

macro_rules! list_clampable_types {
    ($($source:ty),+) => {
        $(
            impl IsClampable for $source {}
        )+
    }
}
list_clampable_types!(f64, f32);

/// Create a transformation struct representing a clamp.
pub fn make_clamp<T>(
    input_properties: Domain<T>, lower: T, upper: T,
) -> Result<Transformation<T, T>, &'static str>
    where T: 'static + PartialOrd + PartialMax + PartialMin + Clone + Debug + IsClampable {
    if lower > upper {
        return Err("lower must not be greater than upper");
    }

    let mut output_properties = input_properties.clone();
    output_properties.bounds = Some(Bounds::<T>::Continuous {
        lower: Some(lower.clone()),
        upper: Some(upper.clone()),
    });

    Ok(Transformation {
        input_domain: input_properties,
        output_domain: output_properties,
        stability: 1.0,
        function: Box::new(move |data: Vec<T>|
            data.into_iter()
                .map(|v| v.partial_min(&upper)?.partial_max(&lower))
                .collect()),
    })
}