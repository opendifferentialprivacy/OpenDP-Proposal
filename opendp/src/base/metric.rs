use std::cmp::Ordering;
use std::ops::{Sub};

use crate::Error;

#[derive(Clone)]
pub enum Metric {
    Symmetric, Hamming, L1Sensitivity, L2Sensitivity
}

#[derive(Clone, PartialEq)]
pub enum Distance<T: PartialOrd + Clone> {
    Symmetric(T), Hamming(T), L1Sensitivity(T), L2Sensitivity(T)
}

#[derive(Clone)]
pub enum Measure {
    Approximate, ZConcentrated
}

#[derive(Clone, PartialEq)]
pub enum Size<T: PartialOrd + Clone> {
    Approximate(T, T), ZConcentrated(T)
}

impl<T: Sub<Output=T> + Clone + PartialOrd> Sub for Size<T> {
    type Output = Result<Size<T>, Error>;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Size::Approximate(eps_l, del_l), Size::Approximate(eps_r, del_r)) =>
                Ok(Size::Approximate(eps_l - eps_r, del_l - del_r)),
            (Size::ZConcentrated(rho_l), Size::ZConcentrated(rho_r)) =>
                Ok(Size::ZConcentrated(rho_l - rho_r)),
            _ => Err(Error::PrivacyMismatch)
        }
    }
}
impl<T: PartialOrd + PartialEq + Clone> PartialOrd for Distance<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use Distance::*;
        match (self, other) {
            (Symmetric(l), Symmetric(r)) => l.partial_cmp(r),
            (Hamming(l), Hamming(r)) => l.partial_cmp(r),
            (L1Sensitivity(l), L1Sensitivity(r)) => l.partial_cmp(&r),
            (L2Sensitivity(l), L2Sensitivity(r)) => l.partial_cmp(&r),
            _ => None
        }
    }
}

impl<T: PartialOrd + PartialEq + Clone> PartialOrd for Size<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use Size::*;
        match (self, other) {
            (Approximate(l_eps, l_del), Approximate(r_eps, r_del)) => {
                if l_eps > r_eps {
                    Some(Ordering::Greater)
                } else {
                    l_del.partial_cmp(r_del)
                }
            }
            (ZConcentrated(l), ZConcentrated(r)) => l.partial_cmp(&r),
            _ => None
        }
    }
}