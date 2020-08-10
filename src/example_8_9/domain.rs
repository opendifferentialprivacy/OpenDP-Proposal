use std::fmt::Debug;

// -- generic type key --
//    N: numeric
//    C: categorical
//
//    NI: numeric input
//    CI: continuous input
//    NO: numeric output
//    CO: continuous output


pub(crate) struct NumericDomain<N>
    where N: PartialOrd + Clone + Debug {
    pub(crate) lower: Option<N>,
    pub(crate) upper: Option<N>,
    pub(crate) optional: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct CategoricalDomain<C>
    where C: Eq + Clone + Debug {
    pub(crate) categories: Option<Vec<C>>,
    pub(crate) optional: bool,
}

#[derive(Clone, Debug)]
pub(crate) enum AtomicDomain<N, C>
    where N: PartialOrd + Clone + Debug,
          C: Eq + Clone + Debug {
    Numeric(NumericDomain<N>),
    Categorical(CategoricalDomain<C>),
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


impl<N> NumericDomain<N>
    where N: PartialOrd + Clone + Debug {
    // TODO
    fn is_valid(&self, x: N) -> bool {
        true
    }
}

impl<C> CategoricalDomain<C>
    where C: Eq + Clone + Debug {
    // TODO
    fn is_valid(&self, x: C) -> bool {
        true
    }
}