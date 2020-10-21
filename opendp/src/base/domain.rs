
use crate::Error;
use crate::base::functions as fun;
use opendp_derive::{AutoFrom, AutoGet};
use std::marker::PhantomData;
use std::any::Any;

// Ethan: Am I correct in understanding this?
// 1. Each domain has an atomic type (the type of the data it contains), a length (the number of such
//    items in the domain, and a bool is_nonempty, which states whether it contains any data.
// 2. What is the need for is_nonempty? Wouldn't a length of 0 => empty?

// Mike:
// 1. Pretty much. Dataframes may contain multiple atomic types- one for each column.
// 2. The separate boolean came from a common situation in Whitenoise where a component
//        would throw an error at function evaluation when the data was empty,
//        but the component didn't need to know the number of records.
//        In this library, we could either
//         A. keep that property, to prevent construction of a function that may be ill-defined
//         B. in some cases propagate optional types, if emptiness would cause an error.
//            Sum- non optional, Mean- optional, Variance- optional, etc.
//            Mechanisms- take optional or non-optional, and sample from propagated bounds if null?
// There are other domain descriptors we will need to pull from Whitenoise. These are just the first ones I grabbed.
pub trait Domain<T> {
    fn as_any(&self) -> &dyn Any;
    fn is_same(&self, other: Box<dyn Domain<T>>) -> bool;
}

#[derive(PartialEq, Clone, Debug)]
pub struct ScalarDomain<T> {
    pub may_have_nullity: bool,
    pub nature: Nature<T>
}

#[derive(PartialEq, Clone, Debug)]
pub struct VectorDomain<C, T> {
    pub atomic_type: Box<dyn Domain<T>>,
    pub is_nonempty: bool,
    pub length: Option<usize>,
    pub container: PhantomData<C>
}

// #[derive(PartialEq, Clone, Debug)]
// pub struct DataframeDomain {
//     pub columns: IndexMap<String, Domain>,
//     pub is_nonempty: bool,
//     pub length: Option<usize>,
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
#[derive(PartialEq, Clone, Debug, AutoFrom, AutoGet)]
pub enum Nature<T> {
    Numeric(Interval<T>),
    Categorical(Categories<T>),
}

// Ethan: What will this be used for?
// Mike: You may want to clamp to a predefined set of categories,
//          and then partition a dataframe or matrix by the categories
//              (you can't just partition by a non-clamped categorical column, because the dimensionality of a parallel release gives the non-dp count of distinct values)
//          or count the number of records in each category
//              (non-stability histograms with public categories)
#[derive(Clone, Debug, PartialEq)]
pub struct Categories<T>(Vec<T>);

impl<T: Eq> Categories<T> {
    pub fn new(values: Vec<T>) -> Categories<T> {
        // TODO: check that the type has equality
        Categories(fun::deduplicate(values).unwrap())
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

// impl DataframeDomain {
//     pub fn assert_non_null(&self) -> Result<(), Error> {
//         for atomic_type in self.columns.values() {
//             atomic_type.assert_non_null()?
//         }
//         Ok(())
//     }
// }

impl<T> ScalarDomain<T> {
    pub fn assert_non_null(&self) -> Result<(), Error> {
        if self.may_have_nullity {
            Err(Error::PotentialNullity)
        } else { Ok(()) }
    }
}
