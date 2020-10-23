use crate::{Error, Measurement, Transformation};
use crate::base::Data;
use crate::base::domain::Domain;
use crate::base::metric::{Distance, Size};

pub fn make_tt_chain<DI, DC, DO, MI, MC, MO>(
    trans_2: Transformation<DC, DO, MC, MO>, trans_1: Transformation<DI, DC, MI, MC>,
    hint: Option<Box<dyn Fn(&Distance<MI>, &Distance<MO>) -> Result<Distance<MC>, Error>>>,
) -> Result<Transformation<DI, DO, MI, MO>, Error>
    where DI: 'static + Domain, DC: 'static + Domain, DO: 'static + Domain,
          MI: 'static + PartialOrd + Clone,
          MC: 'static + PartialOrd + Clone,
          MO: 'static + PartialOrd + Clone {

    if trans_2.input_domain != trans_1.output_domain {
        return Err(crate::Error::DomainMismatch)
    }

    let Transformation {
        input_domain: trans_1_input_domain,
        input_metric: trans_1_input_metric,
        stability_relation: trans_1_stability_relation,
        function: trans_1_function,
        ..
    } = trans_1;

    let Transformation {
        output_domain: trans_2_output_domain,
        output_metric: trans_2_output_metric,
        stability_relation: trans_2_stability_relation,
        function: trans_2_function,
        ..
    } = trans_2;

    Ok(Transformation {
        input_domain: trans_1_input_domain,
        input_metric: trans_1_input_metric,
        output_domain: trans_2_output_domain,
        output_metric: trans_2_output_metric,
        stability_relation: Box::new(move |d_in: &Distance<MI>, d_out: &Distance<MO>| {
            let d_mid = (hint.as_ref().unwrap())(d_in, d_out)?;
            Ok((trans_2_stability_relation)(&d_mid, d_out)? && (trans_1_stability_relation)(d_in, &d_mid)?)
        }),
        function: Box::new(move |data: Data<DI::Member>| (trans_2_function)((trans_1_function)(data)?)),
    })
}


pub fn make_mt_chain<DI, DC, DO, MI, MC, MO>(
    meas: Measurement<DC, DO, MC, MO>, trans: Transformation<DI, DC, MI, MC>,
    hint: Option<Box<dyn Fn(&Distance<MI>, &Size<MO>) -> Result<Distance<MC>, Error>>>,
) -> Result<Measurement<DI, DO, MI, MO>, Error>
    where DI: 'static + Domain, DC: 'static + Domain, DO: 'static,
          MI: 'static + PartialOrd + Clone,
          MC: 'static + PartialOrd + Clone,
          MO: 'static + PartialOrd + Clone {
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
        privacy_relation: Box::new(move |d_in: &Distance<MI>, d_out: &Size<MO>| {
            let d_mid = (hint.as_ref().unwrap())(d_in, d_out)?;
            Ok((meas_privacy_relation)(&d_mid, d_out)? && (trans_stability_relation)(d_in, &d_mid)?)
        }),
        function: Box::new(move |data: Data<DI::Member>| (meas_function)((trans_function)(data)?)),
    })
}
