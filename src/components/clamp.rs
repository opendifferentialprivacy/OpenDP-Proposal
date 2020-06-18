use crate::{Float, Integer, Scalar, Transformation, Domain, MultiSet};
use std::cmp::Ordering;

trait PartialMin: PartialOrd + Sized { fn partial_min(&self, other: &Self) -> Result<Self, &'static str>; }

trait PartialMax: PartialOrd + Sized { fn partial_max(&self, other: &Self) -> Result<Self, &'static str>; }

impl PartialMin for Float {
    fn partial_min(&self, other: &Self) -> Result<Self, &'static str> {
        Ok(match self.partial_cmp(other) {
            None => return Err("types may not be null when comparing"),
            Some(Ordering::Less) => *self,
            _ => *other
        })
    }
}

impl PartialMax for Float {
    fn partial_max(&self, other: &Self) -> Result<Self, &'static str> {
        Ok(match self.partial_cmp(other) {
            None => return Err("types may not be null when comparing"),
            Some(Ordering::Greater) => *self,
            _ => *other
        })
    }
}

impl PartialMin for Integer {
    fn partial_min(&self, other: &Self) -> Result<Self, &'static str> { Ok(*self.min(other)) }
}

impl PartialMax for Integer {
    fn partial_max(&self, other: &Self) -> Result<Self, &'static str> { Ok(*self.max(other)) }
}


/// Create a transformation struct representing a clamp.
pub fn make_clamp(lower: Scalar, upper: Scalar) -> Result<Transformation, &'static str> {
    if lower > upper {
        return Err("lower must not be greater than upper")
    }

    /// Clamp potentially null data between lower/upper.
    fn clamper<T: PartialOrd + PartialMax + PartialMin>(
        lower: T, upper: T, data: &Vec<T>,
    ) -> Result<Vec<T>, &'static str> {
        data.iter()
            .map(|v| v.partial_min(&upper)?.partial_max(&lower))
            .collect::<Result<_, _>>()
    }

    Ok(Transformation {
        input_domain: Domain::Continuous { lower: None, upper: None },
        output_domain: Domain::Continuous { lower: Some(lower.clone()), upper: Some(upper.clone()) },
        stability: 1.0,
        function: Box::new(move |data: &MultiSet| {
            Ok(match (lower.clone(), upper.clone(), data) {
                (Scalar::Float(lower), Scalar::Float(upper), MultiSet::Float(data)) =>
                    MultiSet::Float(clamper(lower, upper, data)?),
                (Scalar::Integer(lower), Scalar::Integer(upper), MultiSet::Integer(data)) =>
                    MultiSet::Integer(clamper(lower, upper, data)?),
                _ => return Err("clamp: data needs to be numeric and data/bounds homogeneously typed")
            })
        }),
    })
}