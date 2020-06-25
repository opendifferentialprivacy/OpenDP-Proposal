use crate::{Transformation, Measurement};
use std::fmt::Debug;

pub fn make_tt_chain<T, U, V>(
    trans_2: Transformation<U, V>, trans_1: Transformation<T, U>
) -> Result<Transformation<T, V>, &'static str>
    where T: 'static + Clone + Debug + PartialEq,
          U: 'static + Clone + Debug + PartialEq,
          V: 'static + Clone + Debug + PartialEq {

    if trans_1.output_domain != trans_2.input_domain {
        return Err("TT Chain: domain mismatch")
    }

    Ok(Transformation {
        input_domain: trans_1.input_domain.clone(),
        output_domain: trans_2.output_domain.clone(),
        stability: trans_1.stability * trans_2.stability,
        function: Box::new(move |data| (trans_2.function)((trans_1.function)(data)?)),
    })
}

pub fn make_mt_chain<T, U, V>(
    meas: Measurement<U, V>, trans: Transformation<T, U>
) -> Result<Measurement<T, V>, &'static str>
    where T: 'static + Clone + Debug + PartialEq,
          U: 'static + Clone + Debug + PartialEq,
          V: 'static + Clone + Debug + PartialEq {

    if trans.output_domain != meas.input_domain {
        return Err("MT Chain: domain mismatch")
    }

    Ok(Measurement {
        input_domain: trans.input_domain.clone(),
        privacy_loss: meas.privacy_loss * trans.stability,
        function: Box::new(move |data| (meas.function)((trans.function)(data)?)),
    })
}