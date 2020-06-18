use crate::{Transformation, Measurement};

pub fn make_tt_chain(trans_2: Transformation, trans_1: Transformation) -> Result<Transformation, &'static str> {
    if trans_2.output_domain != trans_1.input_domain {
        return Err("TT Chain: domain mismatch")
    }

    Ok(Transformation {
        input_domain: trans_1.input_domain.clone(),
        output_domain: trans_2.output_domain.clone(),
        stability: trans_1.stability * trans_2.stability,
        function: Box::new(move |data| {
            (trans_2.function)(&(trans_1.function)(data)?)
        }),
    })
}

pub fn make_mt_chain(meas: Measurement, trans: Transformation) -> Result<Measurement, &'static str> {
    if trans.output_domain != meas.input_domain {
        return Err("MT Chain: domain mismatch")
    }

    Ok(Measurement {
        input_domain: trans.input_domain.clone(),
        privacy_loss: meas.privacy_loss * trans.stability,
        function: Box::new(move |data| {
            (meas.function)(&(trans.function)(data)?)
        }),
    })
}