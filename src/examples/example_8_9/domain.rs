use std::fmt::Debug;
// change: domain representation
//     enum of natures containing generic types with specialized trait bounds

// -- generic type key --
//    N: numeric
//    C: categorical

#[derive(PartialEq, Clone, Debug)]
pub struct NumericNature<N>
    where N: PartialOrd + Clone + Debug {
    pub lower: Option<N>,
    pub upper: Option<N>
}

// custom Default impl doesn't require T: Default
impl<N> Default for NumericNature<N>
    where N: PartialOrd + Clone + Debug {
    fn default() -> Self {
        NumericNature { lower: None, upper: None }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum Nature<N, C>
    where N: PartialOrd + Clone + Debug,
          C: Eq + Clone + Debug {
    Unknown,
    Numeric(NumericNature<N>),
    Categorical(Option<Vec<C>>),
    Boolean
}

#[derive(PartialEq, Clone, Debug)]
pub struct AtomicDomain<N, C>
    where N: PartialOrd + Clone + Debug,
          C: Eq + Clone + Debug {
    pub nature: Nature<N, C>,
    pub nullity: bool,
}

#[derive(PartialEq, Clone, Debug)]
pub enum DataDomain<N, C>
    where N: PartialOrd + Clone + Debug,
          C: Eq + Clone + Debug {
    Scalar(AtomicDomain<N, C>),
    Vector {
        length: Option<i64>,
        is_nonempty: bool,
        atomic_type: AtomicDomain<N, C>,
    },
}
