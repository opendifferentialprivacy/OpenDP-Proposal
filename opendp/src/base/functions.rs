use std::ops::{Mul, Add, Sub, Div};
use num::{CheckedMul, CheckedAdd, CheckedDiv, CheckedSub};
use crate::Error;
use std::hash::Hash;
use itertools::Itertools;
use std::cmp::Ordering;

macro_rules! define_generic {
    ($trait_name:ident, $trait_fun:ident, $error:expr) => {
        pub(crate) fn $trait_fun<T: $trait_name<Output=T>>(l: T, r: T) -> Result<T, Error> {
            l.$trait_fun(&r).ok_or_else(|| $error)
        }
    }
}
define_generic!(CheckedAdd, checked_add, Error::Overflow);
define_generic!(CheckedSub, checked_sub, Error::Overflow);
define_generic!(CheckedMul, checked_mul, Error::Overflow);
define_generic!(CheckedDiv, checked_div, Error::Overflow);

macro_rules! define_generic_infallible {
    ($trait_name:ident, $trait_fun:ident) => {
        pub(crate) fn $trait_fun<T: $trait_name<Output=T>>(l: T, r: T) -> Result<T, Error> {
            Ok(l.$trait_fun(r))
        }
    }
}
define_generic_infallible!(Add, add);
define_generic_infallible!(Sub, sub);
define_generic_infallible!(Mul, mul);
define_generic_infallible!(Div, div);

pub(crate) fn max<T: PartialOrd>(l: T, r: T) -> Result<T, Error> {
    match l.partial_cmp(&r) {
        Some(Ordering::Less) => Ok(r),
        Some(Ordering::Greater) | Some(Ordering::Equal) => Ok(l),
        None => Err(Error::AtomicMismatch)
    }
}

pub(crate) fn min<T: PartialOrd>(l: T, r: T) -> Result<T, Error> {
    match l.partial_cmp(&r) {
        Some(Ordering::Greater) => Ok(r),
        Some(Ordering::Less) | Some(Ordering::Equal) => Ok(l),
        None => Err(Error::AtomicMismatch)
    }
}

pub(crate) fn deduplicate<T: Eq + Clone + Hash>(values: Vec<T>) -> Result<Vec<T>, Error> {
    Ok(values.into_iter().unique().collect())
}