use crate::example_8_9::{
    Transformation, Measurement, Error,
    constructors::{make_tt_chain, make_mt_chain},
};
use std::ops::Shl;
use std::fmt::Debug;


impl<NI, CI, NM, CM, NO, CO> Shl<Transformation<NI, CI, NM, CM>> for Transformation<NM, CM, NO, CO>
    where NI: 'static + PartialOrd + Clone + Debug,
          CI: 'static + Eq + Clone + Debug,
          NM: 'static + PartialOrd + Clone + Debug,
          CM: 'static + Eq + Clone + Debug,
          NO: 'static + PartialOrd + Clone + Debug,
          CO: 'static + Eq + Clone + Debug {
    type Output = Result<Transformation<NI, CI, NO, CO>, Error>;

    fn shl(self, rhs: Transformation<NI, CI, NM, CM>) -> Result<Transformation<NI, CI, NO, CO>, Error> {
        make_tt_chain(self, rhs, None)
    }
}

impl<NI, CI, NM, CM> Shl<Transformation<NI, CI, NM, CM>> for Measurement<NM, CM>
    where NI: 'static + PartialOrd + Clone + Debug,
          CI: 'static + Eq + Clone + Debug,
          NM: 'static + PartialOrd + Clone + Debug,
          CM: 'static + Eq + Clone + Debug {
    type Output = Result<Measurement<NI, CI>, Error>;

    fn shl(self, rhs: Transformation<NI, CI, NM, CM>) -> Result<Measurement<NI, CI>, Error> {
        make_mt_chain(self, rhs, None)
    }
}

impl<NI, CI, NM, CM, NO, CO> Shl<Transformation<NI, CI, NM, CM>> for Result<Transformation<NM, CM, NO, CO>, Error>
    where NI: 'static + PartialOrd + Clone + Debug,
          CI: 'static + Eq + Clone + Debug,
          NM: 'static + PartialOrd + Clone + Debug,
          CM: 'static + Eq + Clone + Debug,
          NO: 'static + PartialOrd + Clone + Debug,
          CO: 'static + Eq + Clone + Debug {
    type Output = Result<Transformation<NI, CI, NO, CO>, Error>;

    fn shl(self, rhs: Transformation<NI, CI, NM, CM>) -> Result<Transformation<NI, CI, NO, CO>, Error> {
        match self {
            Ok(lhs) => lhs << rhs,
            Err(e) => Err(e)
        }
    }
}

impl<NI, CI, NM, CM> Shl<Transformation<NI, CI, NM, CM>> for Result<Measurement<NM, CM>, Error>
    where NI: 'static + PartialOrd + Clone + Debug,
          CI: 'static + Eq + Clone + Debug,
          NM: 'static + PartialOrd + Clone + Debug,
          CM: 'static + Eq + Clone + Debug {
    type Output = Result<Measurement<NI, CI>, Error>;

    fn shl(self, rhs: Transformation<NI, CI, NM, CM>) -> Result<Measurement<NI, CI>, Error> {
        match self {
            Ok(lhs) => lhs << rhs,
            Err(e) => Err(e)
        }
    }
}