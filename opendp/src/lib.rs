// mod examples;
#![allow(dead_code)]

use crate::base::Data;
use crate::base::domain::Domain;
use crate::base::metric::{Metric, Distance, Size};

#[macro_use]
pub mod base;
pub mod constructors;
pub mod ffi;



/// All possible errors in the library
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("{1}")]
    Default(#[source] std::io::Error, &'static str),
    #[error("Domain mismatch")]
    DomainMismatch,
    #[error("Atomic Mismatch")]
    AtomicMismatch,
    #[error("Privacy Mismatch")]
    PrivacyMismatch,
    #[error("Distance Mismatch")]
    DistanceMismatch,
    #[error("Invalid Domain")]
    InvalidDomain,
    #[error("Unsupported Cast")]
    UnsupportedCast,
    #[error("Overflow")]
    Overflow,
    #[error("Insufficient Budget")]
    InsufficientBudget,
    #[error("Unknown Bound")]
    UnknownBound,
    #[error("Potential Nullity")]
    PotentialNullity,
    #[error("{0}")]
    Raw(String),
    #[error("Not Implemented")]
    NotImplemented
}

pub struct Transformation<DI, DO, MI, MO>
    where DI: Domain, DO: Domain,
          MI: PartialOrd + Clone, MO: PartialOrd + Clone {
    pub(crate) input_domain: DI,
    pub(crate) input_metric: Metric,
    pub(crate) output_domain: DO,
    pub(crate) output_metric: Metric,
    pub(crate) stability_relation: Box<dyn Fn(&Distance<MI>, &Distance<MO>) -> Result<bool, Error>>,
    pub(crate) function: Box<dyn Fn(Data<DI::Member>) -> Result<Data<DO::Member>, Error>>,
}

pub struct Measurement<DI, DO, MI, MO>
    where DI: Domain,
          MI: PartialOrd + Clone, MO: PartialOrd + Clone {
    pub(crate) input_domain: DI,
    // pub(crate) input_metric: Metric,
    pub(crate) privacy_relation: Box<dyn Fn(&Distance<MI>, &Size<MO>) -> Result<bool, Error>>,
    pub(crate) function: Box<dyn Fn(Data<DI::Member>) -> Result<Data<DO>, Error>>
}

impl<DI, DO, MI, MO> Transformation<DI, DO, MI, MO>
    where DI: Domain, DO: Domain, MI: PartialOrd + Clone, MO: PartialOrd + Clone {
    pub fn input_domain(&self) -> &DI {
        &self.input_domain
    }
    pub fn output_domain(&self) -> &DO {
        &self.output_domain
    }
}

impl<DI, DO, MI, MO> Measurement<DI, DO, MI, MO>
    where DI: Domain, MI: PartialOrd + Clone, MO: PartialOrd + Clone {
    pub fn function(
        &self, data: Data<DI::Member>, in_dist: &Distance<MI>, out_size: &Size<MO>
    ) -> Result<Data<DO>, Error> {

        if !self.privacy_relation(in_dist, out_size)? {
            return Err(Error::InsufficientBudget)
        }
        (self.function)(data)
    }

    pub fn privacy_relation(&self, in_dist: &Distance<MI>, out_size: &Size<MO>) -> Result<bool, Error> {
        (self.privacy_relation)(in_dist, out_size)
    }
}

// pub struct InteractiveMeasurement<DI, DO, MI, MO>
//     where DI: Domain, MI: PartialOrd + Clone, MO: PartialOrd + Clone {
//     pub(crate) input_domain: DI,
//     pub(crate) input_distance: Distance<MI>,
//     pub(crate) privacy_loss: Size<MO>,
//     pub(crate) function: Box<dyn Fn(Data<DI::Member>) -> Queryable<DI, DO, MI, MO, (Data<DO>, Size<MO>)>>
// }
//
// pub struct Queryable<DI, DO, MI, MO, S>
//     where DI: Domain, MI: PartialOrd + Clone, MO: PartialOrd + Clone {
//     pub(crate) state: S,
//     pub(crate) eval: Box<dyn Fn(Measurement<DI, DO, MI, MO>, Size<MO>, &S) -> (Result<Data<DO>, Error>, S)>
// }
// impl<DI, DO, MI, MO, S> Queryable<DI, DO, MI, MO, S>
//     where DI: Domain, MI: PartialOrd + Clone, MO: PartialOrd + Clone {
//     fn query(
//         &mut self, measurement: Measurement<DI, DO, MI, MO>, privacy_loss: Size<MO>
//     ) -> Result<Data<DO>, Error> {
//         let (response, state) = (self.eval)(measurement, privacy_loss, &self.state);
//         self.state = state;
//         return response
//     }
// }
//
//
// pub fn make_adaptive_composition<DI, MI, MO>(
//     input_domain: DI,
//     input_distance: Distance<MI>,
//     privacy_budget: Size<MO>,
// ) -> InteractiveMeasurement<DI, DI, MI, MO>
//     where DI: Domain + Clone + PartialEq, DI::Member: Clone,
//           MI: PartialOrd + Clone, MO: PartialOrd + Clone + Sub<Output=MO> {
//
//     InteractiveMeasurement {
//         input_domain: input_domain.clone(),
//         input_distance: input_distance.clone(),
//         privacy_loss: privacy_budget.clone(),
//         function: Box::new(move |data: Data<DI::Member>| -> Queryable<DI, DI, MI, MO, (Data<DI::Member>, Size<MO>)> {
//             let input_domain = input_domain.clone();
//             Queryable {
//                 state: (data, privacy_budget.clone()),
//                 eval: Box::new(move |
//                     // query
//                     query: Measurement<DI, DI::Member, MI, MO>,
//                     privacy_loss: Size<MO>,
//                     // state
//                     (data, privacy_budget): &(Data<DI::Member>, Size<MO>)
//                 | -> (Result<Data<DI::Member>, Error>, (Data<DI::Member>, Size<MO>)) {
//                     if query.input_domain != input_domain.clone() {
//                         return (Err(Error::DomainMismatch), (data.clone(), privacy_budget.clone()))
//                     }
//                     // check if privacy budget is sufficient
//                     match privacy_budget.partial_cmp(&privacy_loss) {
//                         // if privacy budget is not comparable with usage, error
//                         None => (Err(Error::PrivacyMismatch), (data.clone(), privacy_budget.clone())),
//                         // if privacy budget is sufficient, apply the query
//                         Some(Ordering::Greater) => match privacy_budget.clone() - privacy_loss {
//                             Ok(new_budget) => ((query.function)(data.clone()), (data.clone(), new_budget)),
//                             Err(e) => (Err(e), (data.clone(), privacy_budget.clone()))
//                         },
//                         // if privacy budget is insufficient, error
//                         _ => (Err(Error::InsufficientBudget), (data.clone(), privacy_budget.clone()))
//                     }
//                 })
//             }
//         })
//     }
// }
//
//
// // issue: state is hardcoded, not generic
// pub fn postprocess<DI, DO, MI, MO>(
//     interactive_measurement: InteractiveMeasurement<DI, DO, MI, MO>,
//     queryable_map: Box<dyn Fn(Queryable<DI, DO, MI, MO, (Data<DI::Member>, Size<MO>)>) -> Queryable<DI, DO, MI, MO, (Data<DI::Member>, Size<MO>)>>
// ) -> InteractiveMeasurement<DI, DO, MI, MO>
//     where DI: Domain, DO: Domain,
//           MI: PartialOrd + Clone, MO: PartialOrd + Clone {
//
//     let function = interactive_measurement.function;
//     InteractiveMeasurement {
//         input_domain: interactive_measurement.input_domain,
//         input_distance: interactive_measurement.input_distance,
//         privacy_loss: interactive_measurement.privacy_loss,
//         function: Box::new(move |data: Data<DI::Member>| {
//             let queryable_inner = (*function)(data);
//             queryable_map(queryable_inner)
//         })
//     }
// }