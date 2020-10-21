use crate::{Transformation, Measurement, Error};
use crate::base::metric::{DataDistance, PrivacyDistance};
use crate::base::Data;


pub fn make_tt_chain<I, C, O>(
    trans_2: Transformation<C, O>, trans_1: Transformation<I, C>,
    hint: Option<Box<dyn Fn(&DataDistance, &DataDistance) -> Result<DataDistance, Error>>>,
) -> Result<Transformation<I, O>, Error> {
    if trans_2.input_domain != trans_1.output_domain {
        return Err(crate::Error::DomainMismatch)
    }

    let Transformation {
        input_domain: trans_input_domain,
        stability_relation: trans_stability_relation,
        function: trans_function,
        ..
    } = trans_1;

    let Transformation {
        output_domain: trans_2_output_domain,
        stability_relation: trans_2_stability_relation,
        function: trans_2_function,
        ..
    } = trans_2;

    Ok(Transformation {
        input_domain: trans_input_domain,
        output_domain: trans_2_output_domain,
        stability_relation: Box::new(move |d_in: &DataDistance, d_out: &DataDistance| {
            let d_mid = (hint.as_ref().unwrap())(d_in, d_out)?;
            Ok((trans_2_stability_relation)(&d_mid, d_out)? && (trans_stability_relation)(d_in, &d_mid)?)
        }),
        function: Box::new(move |data: Data<I>| (trans_2_function)((trans_function)(data)?)),
    })
}


pub fn make_mt_chain<I, C>(
    meas: Measurement<C>, trans: Transformation<I, C>,
    hint: Option<Box<dyn Fn(&DataDistance, &PrivacyDistance) -> Result<DataDistance, Error>>>,
) -> Result<Measurement<C>, Error> {
    if meas.input_domain != trans.output_domain {
        return Err(crate::Error::DomainMismatch)
    }

    let Transformation {
        input_domain: trans_input_domain,
        stability_relation: trans_stability_relation,
        function: trans_function,
        ..
    } = trans;

    let Measurement {
        privacy_relation: meas_privacy_relation,
        function: meas_function,
        ..
    } = meas;

    Ok(Measurement {
        input_domain: trans_input_domain,
        privacy_relation: Box::new(move |d_in: &DataDistance, d_out: &PrivacyDistance| {
            let d_mid = (hint.as_ref().unwrap())(d_in, d_out)?;
            Ok((meas_privacy_relation)(&d_mid, d_out)? && (trans_stability_relation)(d_in, &d_mid)?)
        }),
        function: Box::new(move |data: Data<I>| (meas_function)((trans_function)(data)?)),
    })
}
