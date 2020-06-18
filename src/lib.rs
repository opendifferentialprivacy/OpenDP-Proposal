pub mod components;
mod utilities;

// plug elementary types to set the bit depth
type Integer = i64;
type Float = f64;

type PrivacyLoss = f64;
// enum PrivacyLoss {
//     Approximate { epsilon: f64, delta: f64 },
//     Concentrated { mu: f64, tao: f64 },
//     ZeroConcentrated { ksi: f64, rho: f64 }
// }

#[derive(Clone, Debug, PartialEq)]
pub enum MultiSet {
    Float(Vec<Float>),
    Integer(Vec<Integer>),
    Bool(Vec<bool>),
    String(Vec<String>),
}

// could we drop this in favor of always using the MultiSet type
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Scalar {
    Float(Float),
    Integer(Integer),
    Bool(bool),
    String(String),
}

macro_rules! gen_variant_impls {
    ($source:ty, $variant:ident) => {
        impl From<$source> for Scalar {
            fn from(value: $source) -> Scalar {
                use Scalar::*;
                $variant(value)
            }
        }

        impl From<Vec<$source>> for MultiSet {
            fn from(value: Vec<$source>) -> MultiSet {
                use MultiSet::*;
                $variant(value)
            }
        }
    }
}

gen_variant_impls!(Float, Float);
gen_variant_impls!(Integer, Integer);
gen_variant_impls!(bool, Bool);
gen_variant_impls!(String, String);

pub struct Measurement {
    // QUESTION: how to support measurements that output multiple values, EG tau-threshold categories
    pub function: Box<dyn Fn(&MultiSet) -> Result<Scalar, &'static str>>,
    pub input_domain: Domain,
    pub privacy_loss: PrivacyLoss,
}

pub struct Transformation {
    pub function: Box<dyn Fn(&MultiSet) -> Result<MultiSet, &'static str>>,
    pub input_domain: Domain,
    pub output_domain: Domain,
    pub stability: Float,
}

// This corresponds to the Nature property in whitenoise.
// QUESTION: do you intend Domain to also include nullity, shapes, is_not_empty, dimensionality, etc?
#[derive(Clone, Debug, PartialEq)]
pub enum Domain {
    Continuous { lower: Option<Scalar>, upper: Option<Scalar> },
    Categorical { categories: Option<MultiSet> },
}


