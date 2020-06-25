use std::fmt::Debug;

pub mod components;
pub mod traits;

type PrivacyLoss = f64;
// enum PrivacyLoss {
//     Approximate { epsilon: f64, delta: f64 },
//     Concentrated { mu: f64, tao: f64 },
//     ZeroConcentrated { ksi: f64, rho: f64 }
// }

pub struct Measurement<T: Clone + Debug + PartialEq, U: Clone + Debug + PartialEq> {
    pub function: Box<dyn Fn(Vec<T>) -> Result<U, &'static str>>,
    pub input_properties: Properties<T>,
    pub privacy_loss: PrivacyLoss,
}

pub struct Transformation<T: Clone + Debug + PartialEq, U: Clone + Debug + PartialEq> {
    pub function: Box<dyn Fn(Vec<T>) -> Result<Vec<U>, &'static str>>,
    pub input_properties: Properties<T>,
    pub output_properties: Properties<U>,
    pub stability: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Domain<T: Clone + Debug + PartialEq> {
    Continuous { lower: Option<T>, upper: Option<T> },
    Categorical { categories: Option<Vec<T>> },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Properties<T: Clone + Debug + PartialEq> {
    pub has_nullity: bool,
    pub domain: Option<Domain<T>>
}
