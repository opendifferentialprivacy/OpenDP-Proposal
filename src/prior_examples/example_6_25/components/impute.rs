use crate::example_7_1::{Transformation, Domain};
use std::fmt::Debug;
use crate::example_7_1::traits::{IsNull, GenUniform};

/// Create a transformation struct representing a clamp.
pub fn make_imputation<T>(
    input_properties: Domain<T>, lower: T, upper: T
) -> Result<Transformation<T, T>, &'static str>
    where T: 'static + PartialOrd + Clone + Debug + IsNull + GenUniform {
    if lower > upper {
        return Err("lower must not be greater than upper")
    }

    let mut output_properties = input_properties.clone();
    output_properties.has_nullity = false;

    Ok(Transformation {
        input_domain: input_properties,
        output_domain: output_properties,
        stability: 1.0,
        function: Box::new(move |data: Vec<T>|
            data.into_iter().map(|v|
                if v.is_null() {
                    T::sample_uniform(lower.clone(), upper.clone())
                } else {
                    Ok(v)
                }).collect()),
    })
}