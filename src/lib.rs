use std::fmt::Debug;
use std::ops::Add;

mod example_7_30_compiler_validation;

mod example_8_1_descriptors;

mod example_8_3_1_atomic_domain;
mod example_8_3_2_atomic_domain_trait;

mod example_8_4_1_data_domain_enum;
mod example_8_4_2_metrics;
mod example_8_4_3_trait_relations;
mod example_8_4_4_enum_relations;
mod example_8_4_5_queryable;

pub mod components;
pub mod traits;


struct LaplaceMechanism {}
struct Sum {}
struct Clamp {}
struct MTChain {}
struct TTChain {}

enum Descriptor {
    LaplaceMechanism(LaplaceMechanism),
    Sum(Sum),
    Clamp(Clamp),
    MTChain(MTChain),
    TTChain(TTChain)
}

enum Distance {
    Hamming(u32),
    Symmetric(u32),
    L1(f64),
    L2(f64),
    zCDP(f64),
    ApproxDP(f64, f64),
}

impl Distance {
    pub fn metric(&self) -> Metric {
        match self {
            Distance::Hamming(_) => Metric::Hamming,
            Distance::L1(_) => Metric::L1,
            Distance::L2(_) => Metric::L2,
            Distance::zCDP(_) => Metric::zCDP,
            Distance::ApproxDP(..) => Metric::ApproxDP,
        }
    }
}

enum Metric {
    Hamming,
    Symmetric,
    L1,
    L2,
    zCDP,
    ApproxDP,
}



pub struct Measurement<T: Clone + Debug + PartialEq, U: Clone + Debug + PartialEq> {
    pub function: Box<dyn Fn(Vec<T>) -> Result<U, &'static str>>,
    pub privacy_relation: Box<dyn Fn(Distance, Distance) -> bool>,
    pub input_domain: Domain<T>,
}

pub struct Transformation<T: Clone + Debug + PartialEq, U: Clone + Debug + PartialEq> {
    pub input_domain: Domain<T>,
    pub output_domain: Domain<U>,
    pub function: Box<dyn Fn(Vec<T>) -> Result<Vec<U>, &'static str>>,
    pub privacy_relation: Box<dyn Fn(Distance, Distance) -> bool>,
    pub hint: Box<dyn Fn(Distance, Distance) -> Result<Distance, &'static str>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Bounds<T: Clone + Debug + PartialEq> {
    Continuous { lower: Option<T>, upper: Option<T> },
    Categorical { categories: Option<Vec<T>> },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Domain<T: Clone + Debug + PartialEq> {
    pub has_nullity: bool,
    pub bounds: Option<Bounds<T>>,
}
