use std::fmt::Debug;
use std::ops::{Mul, Sub};

use crate::Error;
use crate::base::value::{NumericScalar, UnsignedIntScalar};

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


#[derive(Clone, PartialOrd, PartialEq)]
pub enum DataDistance {
    Symmetric(UnsignedIntScalar),
    Hamming(UnsignedIntScalar),
    L1Sensitivity(NumericScalar),
    L2Sensitivity(NumericScalar),
}

#[derive(Clone, Debug, PartialOrd, PartialEq)]
pub enum PrivacyDistance {
    Approximate(NumericScalar, NumericScalar),
    ZConcentrated(NumericScalar)
}

impl Mul<NumericScalar> for DataDistance {
    type Output = Result<DataDistance, Error>;

    fn mul(self, rhs: NumericScalar) -> Self::Output {
        Ok(match self {
            DataDistance::Hamming(d) => DataDistance::Hamming((d * rhs)?),
            DataDistance::Symmetric(d) => DataDistance::Symmetric((d * rhs)?),
            DataDistance::L1Sensitivity(d) => DataDistance::L1Sensitivity((d * rhs)?),
            DataDistance::L2Sensitivity(d) => DataDistance::L2Sensitivity((d * rhs)?),
            _ => unimplemented!()
        })
    }
}

impl Sub<&PrivacyDistance> for &PrivacyDistance {
    type Output = Result<PrivacyDistance, Error>;

    fn sub(self, rhs: &PrivacyDistance) -> Self::Output {
        Ok(match (self, rhs) {
            (PrivacyDistance::Approximate(eps_l, del_l), PrivacyDistance::Approximate(eps_r, del_r)) =>
                PrivacyDistance::Approximate((eps_l + eps_r)?, (del_l + del_r)?),
            (PrivacyDistance::ZConcentrated(rho_l), PrivacyDistance::ZConcentrated(rho_r)) =>
                PrivacyDistance::ZConcentrated((rho_l + rho_r)?),
            _ => return Err(Error::PrivacyMismatch)
        })
    }
}
