use crate::example_8_4_1_data_domain_enum::*;
use std::ops::Mul;

trait Metric {
    fn is_single_real(&self) -> bool;
    fn has_upper_bound(&self) -> bool;
    fn is_triangular(&self) -> bool;
    fn has_path_connectivity(&self) -> bool;
    fn is_symmetric(&self) -> bool;
}

#[derive(PartialEq)]
pub(crate) enum DataMetric {
    DistFloat(DistFloat),
    L1(L1),
    L2(L2),
    AddRemove(AddRemove),
    And(AndMetric)
}
pub(crate) enum PrivacyMetric {
    PureDP(PureDP),
    ApproxDP(ApproxDP)
}

struct DistFloat;
struct L1;
struct L2;
struct AddRemove;
struct AndMetric(DataMetric, DataMetric);
struct PureDP;
struct ApproxDP;

impl Metric for DistFloat {
    fn is_single_real(&self) -> bool {
        true
    }

    fn has_upper_bound(&self) -> bool {
        unimplemented!()
    }

    fn is_triangular(&self) -> bool {
        unimplemented!()
    }

    fn has_path_connectivity(&self) -> bool {
        unimplemented!()
    }

    fn is_symmetric(&self) -> bool {
        unimplemented!()
    }
}



pub(crate) enum DataDistance {
    DistFloat(f64),
    L1(f64),
    L2(f64),
    AddRemove(u16),
    And(DataDistance, DataDistance),
}

pub(crate) enum PrivacyDistance {
    PureDP(f64),
    ApproxDP(f64, f64)
}

impl Mul<i64> for DataDistance {
    type Output = DataDistance;

    fn mul(self, rhs: i64) -> Self::Output {
        match self {
            DataDistance::DistFloat(d) => DataDistance::DistFloat(d * rhs),
            DataDistance::L1(d) => DataDistance::L1(d * rhs),
            _ => unimplemented!()
        }
    }
}
