use std::fmt::Debug;

// -- generic type key --
//    N: numeric
//    C: categorical
//
//    NI: numeric input
//    CI: continuous input
//    NO: numeric output
//    CO: continuous output

#[derive(PartialEq, Clone, Debug)]
pub(crate) struct NumericNature<N>
    where N: PartialOrd + Clone + Debug {
    pub(crate) lower: Option<N>,
    pub(crate) upper: Option<N>
}

// custom Default impl doesn't require T: Default
impl<N> Default for NumericNature<N>
    where N: PartialOrd + Clone + Debug {
    fn default() -> Self {
        NumericNature { lower: None, upper: None }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub(crate) enum Nature<N, C>
    where N: PartialOrd + Clone + Debug,
          C: Eq + Clone + Debug {
    Numeric(NumericNature<N>),
    Categorical(Option<Vec<C>>),
    Boolean
}

#[derive(PartialEq, Clone, Debug)]
pub(crate) struct AtomicDomain<N, C>
    where N: PartialOrd + Clone + Debug,
          C: Eq + Clone + Debug {
    pub(crate) nature: Nature<N, C>,
    pub(crate) nullity: bool,
}

#[derive(PartialEq, Clone, Debug)]
pub(crate) enum DataDomain<N, C>
    where N: PartialOrd + Clone + Debug,
          C: Eq + Clone + Debug {
    Scalar(AtomicDomain<N, C>),
    Vector {
        length: Option<i64>,
        is_nonempty: bool,
        atomic_type: AtomicDomain<N, C>,
    },
}
