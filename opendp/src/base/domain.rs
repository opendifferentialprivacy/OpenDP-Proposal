use std::cmp::Ordering;

use indexmap::map::IndexMap;


use opendp_derive::{AutoFrom, AutoGet, apply_categorical, apply_numeric};

use crate::base::value::*;
use crate::Error;
use crate::base::functions as fun;

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
#[derive(PartialEq, Clone, Debug, AutoFrom, AutoGet)]
pub enum Domain {
    Scalar(ScalarDomain),
    Vector(VectorDomain),
    Dataframe(DataframeDomain),
}

#[derive(PartialEq, Clone, Debug)]
pub struct ScalarDomain {
    pub may_have_nullity: bool,
    pub nature: Nature,
}

#[derive(PartialEq, Clone, Debug)]
pub struct VectorDomain {
    pub atomic_type: Box<Domain>,
    pub is_nonempty: bool,
    pub length: Option<usize>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct DataframeDomain {
    pub columns: IndexMap<String, Domain>,
    pub is_nonempty: bool,
    pub length: Option<usize>,
}

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
pub enum Nature {
    Numeric(Interval),
    Categorical(Categories),
}

// Ethan: What will this be used for?
// Mike: You may want to clamp to a predefined set of categories,
//          and then partition a dataframe or matrix by the categories
//              (you can't just partition by a non-clamped categorical column, because the dimensionality of a parallel release gives the non-dp count of distinct values)
//          or count the number of records in each category
//              (non-stability histograms with public categories)
#[derive(Clone, Debug, PartialEq)]
pub struct Categories(Vector);

impl Categories {
    pub fn new(values: Vector) -> Categories {
        // TODO: check that the type has equality
        Categories(apply_categorical!(fun::deduplicate, values: Vector).unwrap())
    }
    pub fn get(self) -> Vector { self.0 }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Interval(Option<Scalar>, Option<Scalar>);

// IMPLEMENTATIONS
impl Interval {
    pub fn new(lower: Option<Scalar>, upper: Option<Scalar>) -> Result<Interval, Error> {
        if let (Some(l), Some(u)) = (&lower, &upper) {
            // TODO: check is numeric
            match apply_numeric!(fun::cmp, l: Scalar, u: Scalar)? {
                None => return Err(crate::Error::AtomicMismatch),
                Some(Ordering::Greater) => return Err(crate::Error::InvalidDomain),
                _ => ()
            }
        }
        Ok(Interval(lower, upper))
    }

    pub fn bounds(self) -> (Option<Scalar>, Option<Scalar>) {
        (self.0, self.1)
    }
}

impl Domain {
    // TODO: should we have domain constructors at this fine granularity?
    pub fn numeric_scalar(
        lower: Option<Scalar>, upper: Option<Scalar>, may_have_nullity: bool,
    ) -> Result<Self, Error> {
        Ok(Domain::Scalar(ScalarDomain {
            may_have_nullity,
            nature: Nature::Numeric(Interval::new(lower, upper)?),
        }))
    }

    pub fn assert_non_null(&self) -> Result<(), Error> {
        Ok(match self {
            Domain::Scalar(domain) => domain.assert_non_null()?,
            Domain::Vector(domain) => domain.atomic_type.assert_non_null()?,
            Domain::Dataframe(domain) => domain.assert_non_null()?,
        })
    }
}


impl DataframeDomain {
    pub fn assert_non_null(&self) -> Result<(), Error> {
        for atomic_type in self.columns.values() {
            atomic_type.assert_non_null()?
        }
        Ok(())
    }
}

impl ScalarDomain {
    pub fn assert_non_null(&self) -> Result<(), Error> {
        if self.may_have_nullity {
            Err(Error::PotentialNullity)
        } else { Ok(()) }
    }
}
