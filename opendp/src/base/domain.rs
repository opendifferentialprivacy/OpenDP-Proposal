
use crate::Error;
use std::marker::PhantomData;
use itertools::Itertools;
use std::hash::Hash;

pub trait Domain: Clone + PartialEq {
    type Member;
    // fn valid(&self, x: Self::Member) -> bool;
}

#[derive(PartialEq, Clone, Debug)]
pub struct ScalarDomain<C: Clone, T: PartialEq + Clone> {
    pub may_have_nullity: bool,
    pub nature: Nature<T>,
    pub container: PhantomData<C>
}

impl<C: Clone + PartialEq, T: PartialEq + Clone> Domain for ScalarDomain<C, T> {
    type Member = C;
}

#[derive(PartialEq, Clone, Debug)]
pub struct VectorDomain<T: Clone, D: Domain + Clone> {
    pub atomic_type: D,
    pub is_nonempty: bool,
    pub length: Option<usize>,
    pub container: PhantomData<T>
}

impl<T: Clone + PartialEq, D: Domain + Clone> Domain for VectorDomain<T, D> {
    type Member = T;
}

// #[derive(PartialEq, Clone, Debug)]
// pub struct DataframeDomain {
//     pub columns: IndexMap<String, Box<dyn Any>>,
//     pub is_nonempty: bool,
//     pub length: Option<usize>,
// }
//
// impl Domain for DataframeDomain {
//     type Member = IndexMap<String, Box<dyn Any>>;
// }

// Ethan: I know I asked this in our meeting -
// is there a particular reason it is called "nature"? Just curious
// This is essentially giving the valid range / valid values for the domain right?
//
// Mike: The terminology doesn't matter to me.
// Variables are occasionally categorized by "nature" but the frequency of that term isn't super high
// Basically referring to these:
// https://en.wikipedia.org/wiki/Statistical_data_type
// Some domain descriptors are only relevant for certain data types
#[derive(PartialEq, Clone, Debug)]
pub enum Nature<T> {
    Numeric(Interval<T>),
    Categorical(Categories<T>),
}

impl<T> Nature<T> {
    pub fn to_numeric(self) -> Result<Interval<T>, Error> {
        match self {
            Nature::Numeric(v) => Ok(v),
            _ => Err(Error::AtomicMismatch)
        }
    }

    pub fn as_numeric(&self) -> Result<&Interval<T>, Error> {
        match self {
            Nature::Numeric(v) => Ok(v),
            _ => Err(Error::AtomicMismatch)
        }
    }

    pub fn to_categorical(self) -> Result<Categories<T>, Error> {
        match self {
            Nature::Categorical(v) => Ok(v),
            _ => Err(Error::AtomicMismatch)
        }
    }

    pub fn as_categorical(&self) -> Result<&Categories<T>, Error> {
        match self {
            Nature::Categorical(v) => Ok(v),
            _ => Err(Error::AtomicMismatch)
        }
    }
}

// Ethan: What will this be used for?
// Mike: You may want to clamp to a predefined set of categories,
//          and then partition a dataframe or matrix by the categories
//              (you can't just partition by a non-clamped categorical column, because the dimensionality of a parallel release gives the non-dp count of distinct values)
//          or count the number of records in each category
//              (non-stability histograms with public categories)
#[derive(Clone, Debug, PartialEq)]
pub struct Categories<T>(Vec<T>);

impl<T: Eq + Clone + Hash> Categories<T> {
    pub fn new(values: Vec<T>) -> Categories<T> {
        Categories(values.into_iter().unique().collect())
    }
    pub fn get(self) -> Vec<T> { self.0 }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Interval<T>(Option<T>, Option<T>);

// IMPLEMENTATIONS
impl<T: PartialOrd> Interval<T> {
    pub fn new(
        lower: Option<T>, upper: Option<T>
    ) -> Result<Interval<T>, Error> {

        if let (Some(l), Some(u)) = (&lower, &upper) {
            if l > u { return Err(crate::Error::InvalidDomain) }
        }
        Ok(Interval(lower, upper))
    }

    pub fn bounds(self) -> (Option<T>, Option<T>) {
        (self.0, self.1)
    }
}

impl<C: Clone, T: PartialEq + Clone> ScalarDomain<C, T> {
    pub fn assert_non_null(&self) -> Result<(), Error> {
        if self.may_have_nullity {
            Err(Error::PotentialNullity)
        } else { Ok(()) }
    }
}
