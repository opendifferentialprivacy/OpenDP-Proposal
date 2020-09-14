use std::cmp::Ordering;
use std::fmt::Debug;
use std::ops::{Add, Mul, Sub};

use crate::base::functions as fun;
use crate::base::value::*;
use crate::Error;

trait MathMetric {
    fn is_single_real(&self) -> bool;
    fn has_upper_bound(&self) -> bool;
    fn is_triangular(&self) -> bool;
    fn has_path_connectivity(&self) -> bool;
    fn is_symmetric(&self) -> bool;
}

#[derive(PartialEq, Clone)]
pub enum Metric {
    Symmetric(Symmetric),
    Hamming(Hamming),
    L1Sensitivity(L1Sensitivity),
    L2Sensitivity(L2Sensitivity),
}

#[derive(PartialEq, Clone)]
pub enum PrivacyMeasure {
    Approximate(ApproximateDP),
    ZConcentrated(ZConcentratedDP),
}

#[derive(Clone, Debug, PartialEq)]
pub struct L1Sensitivity;

#[derive(Clone, Debug, PartialEq)]
pub struct L2Sensitivity;

// substitute
#[derive(Clone, Debug, PartialEq)]
pub struct Symmetric;

// add/remove
#[derive(Clone, Debug, PartialEq)]
pub struct Hamming;

#[derive(Clone, Debug, PartialEq)]
pub struct ApproximateDP;

#[derive(Clone, Debug, PartialEq)]
pub struct ZConcentratedDP;


#[derive(Clone, PartialEq)]
pub enum DataDistance {
    Symmetric(NumericScalar),
    Hamming(NumericScalar),
    L1Sensitivity(NumericScalar),
    L2Sensitivity(NumericScalar),
}

#[derive(Clone, Debug, PartialEq)]
pub enum PrivacyDistance {
    Approximate(NumericScalar, NumericScalar),
    ZConcentrated(NumericScalar)
}

macro_rules! impl_trait_privacy_distance {
    ($trait_name:ident, $trait_fun:ident, $generic_fun:path) => {
        impl $trait_name<PrivacyDistance> for PrivacyDistance {
            type Output = Result<PrivacyDistance, Error>;

            fn $trait_fun(self, rhs: PrivacyDistance) -> Self::Output {
                Ok(match (self, rhs) {
                    (PrivacyDistance::Approximate(eps_l, del_l), PrivacyDistance::Approximate(eps_r, del_r)) =>
                        PrivacyDistance::Approximate(apply_numeric_scalar!($generic_fun, eps_l, eps_r)?, apply_numeric_scalar!($generic_fun, del_l, del_r)?),
                    (PrivacyDistance::ZConcentrated(rho_l), PrivacyDistance::ZConcentrated(rho_r)) =>
                        PrivacyDistance::ZConcentrated(apply_numeric_scalar!($generic_fun, rho_l, rho_r)?),
                    _ => return Err(Error::PrivacyMismatch)
                })
            }
        }
    }
}
impl_trait_privacy_distance!(Add, add, fun::add);
impl_trait_privacy_distance!(Sub, sub, fun::sub);
impl_trait_privacy_distance!(Mul, mul, fun::mul);

impl PartialOrd for PrivacyDistance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use PrivacyDistance::*;
        match (self, other) {
            (Approximate(l_eps, l_del), Approximate(r_eps, r_del)) => {
                if let Some(Ordering::Greater) = apply_numeric_scalar!(fun::cmp, l_eps, r_eps).unwrap() {
                    Some(Ordering::Greater)
                } else {
                    apply_numeric_scalar!(fun::cmp, l_del, r_del).unwrap()
                }
            }

            (ZConcentrated(l), ZConcentrated(r)) => apply_numeric_scalar!(fun::cmp, l, r).unwrap(),
            _ => None
        }
    }
}


impl PartialOrd for DataDistance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use DataDistance::*;
        match (self, other) {
            (Symmetric(l), Symmetric(r)) => apply_numeric_scalar!(fun::cmp, l, r).ok().and_then(|v| v),
            (Hamming(l), Hamming(r)) => apply_numeric_scalar!(fun::cmp, l, r).ok().and_then(|v| v),
            (L1Sensitivity(l), L1Sensitivity(r)) => apply_numeric_scalar!(fun::cmp, l, r).ok().and_then(|v| v),
            (L2Sensitivity(l), L2Sensitivity(r)) => apply_numeric_scalar!(fun::cmp, l, r).ok().and_then(|v| v),
            _ => None
        }
    }
}