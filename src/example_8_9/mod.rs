use crate::example_8_9::metric::{DataMetric, PrivacyMeasure, DataDistance, PrivacyDistance, DistFloat, ApproxDP};
use crate::example_8_9::domain::{DataDomain, Nature, NumericNature, AtomicDomain};
use std::fmt::Debug;
use num::traits::Signed;
use num::Zero;
use std::ops::Shl;

mod domain;
mod metric;

type Error = &'static str;

#[derive(Clone, Debug)]
enum Data {
    Pointer(i64),
    // Literal(Value)
}

struct Transformation<NI, CI, NO, CO>
    where NI: PartialOrd + Clone + Debug,
          CI: Eq + Clone + Debug,
          NO: PartialOrd + Clone + Debug,
          CO: Eq + Clone + Debug {
    input_domain: DataDomain<NI, CI>,
    input_metric: DataMetric,
    output_domain: DataDomain<NO, CO>,
    output_metric: DataMetric,
    stability_relation: Box<dyn Fn(&DataDistance, &DataDistance) -> bool>,
    function: Box<dyn Fn(Data) -> Result<Data, Error>>,
}

struct Measurement<NI, CI>
    where NI: PartialOrd + Clone + Debug,
          CI: Eq + Clone + Debug {
    input_metric: DataMetric,
    input_domain: DataDomain<NI, CI>,
    output_measure: PrivacyMeasure,
    privacy_relation: Box<dyn Fn(&DataDistance, &PrivacyDistance) -> bool>,
    function: Box<dyn Fn(Data) -> Result<Data, Error>>,
}

struct InteractiveMeasurement<NI, CI, S>
    where NI: PartialOrd + Clone + Debug,
          CI: Eq + Clone + Debug {
    input_domain: DataDomain<NI, CI>,
    input_distance: DataDistance,
    privacy_loss: PrivacyDistance,
    function: Box<dyn Fn(Data) -> Queryable<NI, CI, S>>,
}

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

struct Queryable<NI, CI, S>
    where NI: PartialOrd + Clone + Debug,
          CI: Eq + Clone + Debug {
    state: S,
    eval: Box<dyn Fn(Measurement<NI, CI>, PrivacyDistance, &S) -> (Result<Data, Error>, S)>,
}

impl<NI, CI, S> Queryable<NI, CI, S>
    where NI: PartialOrd + Clone + Debug,
          CI: Eq + Clone + Debug {
    fn query(&mut self, measurement: Measurement<NI, CI>, privacy_loss: PrivacyDistance) -> Result<Data, Error> {
        let (response, state) = (self.eval)(measurement, privacy_loss, &self.state);
        self.state = state;
        return response;
    }
}

fn make_adaptive_composition<NI: 'static, CI: 'static>(
    input_domain: DataDomain<NI, CI>,
    input_distance: DataDistance,
    privacy_budget: PrivacyDistance,
) -> InteractiveMeasurement<NI, CI, (Data, PrivacyDistance)>
    where NI: PartialOrd + Clone + Debug,
          CI: Eq + Clone + Debug {
    InteractiveMeasurement {
        input_domain: input_domain.clone(),
        input_distance: input_distance.clone(),
        privacy_loss: privacy_budget.clone(),
        function: Box::new(move |data: Data| -> Queryable<NI, CI, (Data, PrivacyDistance)> {
            let input_domain = input_domain.clone();
            Queryable {
                state: (data, privacy_budget.clone()),
                eval: Box::new(move |
                    // query
                    query: Measurement<NI, CI>,
                    privacy_loss: PrivacyDistance,
                    // state
                    (data, privacy_budget): &(Data, PrivacyDistance)| -> (Result<Data, Error>, (Data, PrivacyDistance)) {
                    if query.input_domain != input_domain.clone() {
                        return (Err("domain mismatch"), (data.clone(), privacy_budget.clone()));
                    }
                    if privacy_budget < &privacy_loss {
                        (Err("insufficient budget"), (data.clone(), privacy_budget.clone()))
                    } else {
                        match privacy_budget - &privacy_loss {
                            Ok(new_budget) => ((query.function)(data.clone()), (data.clone(), new_budget)),
                            Err(e) => (Err(e), (data.clone(), privacy_budget.clone()))
                        }
                    }
                }),
            }
        }),
    }
}

fn postprocess<NI: 'static, CI: 'static, S: 'static>(
    interactive_measurement: InteractiveMeasurement<NI, CI, S>,
    queryable_map: Box<dyn Fn(Queryable<NI, CI, S>) -> Queryable<NI, CI, S>>,
) -> InteractiveMeasurement<NI, CI, S>
    where NI: PartialOrd + Clone + Debug,
          CI: Eq + Clone + Debug {
    let function = interactive_measurement.function;
    InteractiveMeasurement {
        input_domain: interactive_measurement.input_domain,
        input_distance: interactive_measurement.input_distance,
        privacy_loss: interactive_measurement.privacy_loss,
        function: Box::new(move |data: Data| {
            let queryable_inner = (*function)(data);
            queryable_map(queryable_inner)
        }),
    }
}


fn make_row_transform<NI, CI, NO, CO>(
    input_domain: DataDomain<NI, CI>,
    input_metric: DataMetric,
    output_domain: DataDomain<NO, CO>,
    output_metric: DataMetric,
    function: Box<dyn Fn(Data) -> Result<Data, Error>>,
) -> Transformation<NI, CI, NO, CO>
    where NI: PartialOrd + Clone + Debug,
          CI: Eq + Clone + Debug,
          NO: PartialOrd + Clone + Debug,
          CO: Eq + Clone + Debug {
    Transformation {
        input_domain,
        input_metric,
        output_domain,
        // cannot make any claim of c_stability for arbitrary functions
        output_metric,
        stability_relation: Box::new(move |_in_dist: &DataDistance, _out_dist: &DataDistance| -> bool { true }),
        function,
    }
}

fn make_clamp_numeric<NI, CI>(
    input_domain: DataDomain<NI, CI>,
    input_metric: DataMetric,
    lower: NI, upper: NI,
) -> Result<Transformation<NI, CI, NI, CI>, Error>
    where NI: PartialOrd + Clone + Debug,
          CI: Eq + Clone + Debug {
    let output_domain = match &input_domain {
        DataDomain::Vector {
            atomic_type,
            is_nonempty,
            length
        } => {
            // rest/unpack the prior numeric domain descriptors
            let AtomicDomain { nature, nullity } = atomic_type;
            let NumericNature {
                lower: prior_lower, upper: prior_upper
            } = if let Nature::Numeric(v) = nature { v } else {
                return Err("invalid atomic type");
            };

            // construct a new domain with updated bounds
            DataDomain::Vector {
                length: length.clone(),
                is_nonempty: *is_nonempty,
                atomic_type: AtomicDomain {
                    nature: Nature::Numeric(NumericNature {
                        lower: Some(prior_lower.as_ref()
                            .map(|l| if l < &lower { &lower } else { l })
                            .unwrap_or(&lower).clone()),
                        upper: Some(prior_upper.as_ref()
                            .map(|u| if u > &upper { &upper } else { u })
                            .unwrap_or(&upper).clone()),
                    }),
                    nullity: *nullity,
                },
            }
        }
        _ => return Err("invalid input domain")
    };

    Ok(Transformation {
        input_domain,
        input_metric: input_metric.clone(),
        output_domain,
        output_metric: input_metric,
        stability_relation: Box::new(move |in_dist, out_dist| in_dist <= out_dist),
        // issue: how to differentiate between calls out to different execution environments
        function: Box::new(move |data| Ok(data)),
    })
}


fn make_tt_chain<NI, CI, NM, CM, NO, CO>(
    trans_2: Transformation<NM, CM, NO, CO>,
    trans_1: Transformation<NI, CI, NM, CM>,
    hint: Option<Box<fn(&DataDistance, &DataDistance) -> DataDistance>>,
) -> Result<Transformation<NI, CI, NO, CO>, Error>
    where NI: 'static + PartialOrd + Clone + Debug,
          CI: 'static + Eq + Clone + Debug,
          NM: 'static + PartialOrd + Clone + Debug,
          CM: 'static + Eq + Clone + Debug,
          NO: 'static + PartialOrd + Clone + Debug,
          CO: 'static + Eq + Clone + Debug {

    if trans_1.output_domain != trans_2.input_domain {
        return Err("TT chain: domain mismatch");
    }

    if trans_1.output_metric != trans_2.input_metric {
        return Err("TT chain: metric mismatch");
    }

    // destructure to avoid "move"ing entire structs into closures
    let Transformation {
        stability_relation: trans_1_stability,
        function: trans_1_function, ..
    } = trans_1;
    let Transformation {
        stability_relation: trans_2_stability,
        function: trans_2_function, ..
    } = trans_2;

    Ok(Transformation {
        input_domain: trans_1.input_domain,
        input_metric: trans_1.input_metric,
        output_domain: trans_2.output_domain,
        output_metric: trans_2.output_metric,
        stability_relation: Box::new(move |dist_in, dist_out| {
            let dist_mid = match hint.as_ref() {
                Some(hint) => (*hint)(dist_in, dist_out),
                // TODO: binary search
                None => return false
            };
            (trans_2_stability)(dist_in, &dist_mid) && (trans_1_stability)(&dist_mid, dist_out)
        }),
        function: Box::new(move |data| (trans_2_function)((trans_1_function)(data)?)),
    })
}


fn make_mt_chain<NI, CI, NM, CM>(
    meas: Measurement<NM, CM>,
    trans: Transformation<NI, CI, NM, CM>,
    hint: Option<Box<fn(&DataDistance, &PrivacyDistance) -> DataDistance>>,
) -> Result<Measurement<NI, CI>, Error>
    where NI: 'static + PartialOrd + Clone + Debug,
          CI: 'static + Eq + Clone + Debug,
          NM: 'static + PartialOrd + Clone + Debug,
          CM: 'static + Eq + Clone + Debug {

    if trans.output_domain != meas.input_domain {
        return Err("MT chain: domain mismatch");
    }
    if trans.output_metric != meas.input_metric {
        return Err("MT chain: metric mismatch");
    }
    // destructure to avoid moving entire structs into closures
    let Transformation {
        stability_relation: trans_relation,
        function: trans_function, ..
    } = trans;
    let Measurement {
        privacy_relation: meas_relation,
        function: meas_function, ..
    } = meas;

    Ok(Measurement {
        input_domain: trans.input_domain,
        input_metric: trans.input_metric,
        output_measure: meas.output_measure,
        privacy_relation: Box::new(move |dist_in, dist_out| {
            let dist_mid = match hint.as_ref() {
                Some(hint) => (*hint)(dist_in, dist_out),
                // TODO: binary search
                None => return false
            };
            (trans_relation)(dist_in, &dist_mid) && (meas_relation)(&dist_mid, dist_out)
        }),
        function: Box::new(move |data| (meas_function)((trans_function)(data)?)),
    })
}


fn make_sum<NI: 'static, CI>(
    input_domain: DataDomain<NI, CI>,
    input_metric: DataMetric,
) -> Result<Transformation<NI, CI, NI, CI>, Error>
    where NI: PartialOrd + Clone + Debug + Signed,
          CI: Eq + Clone + Debug,
          f64: From<NI> {

    let AtomicDomain {nullity, nature} = if let DataDomain::Vector {
        atomic_type, ..
    } = &input_domain { atomic_type } else {return Err("Sum: input must be a vector")};

    let (lower, upper) = if let Nature::Numeric(NumericNature{
        lower: Some(lower), upper: Some(upper)}) = nature {(lower.clone(), upper.clone())} else {
        return Err("Sum: input nature must be numeric")
    };

    Ok(Transformation {
        output_domain: DataDomain::Scalar(AtomicDomain {
            nature: Nature::Numeric(NumericNature::default()),
            nullity: *nullity
        }),
        input_domain,
        input_metric,
        output_metric: DataMetric::DistFloat(DistFloat {}),
        stability_relation: Box::new(move |dist_in, dist_out|
            match (dist_in, dist_out) {
                (DataDistance::AddRemove(dist_in), DataDistance::DistFloat(dist_out)) =>
                    *dist_in as f64 * (f64::from(if lower.abs() < upper.abs() { upper.abs() } else { lower.abs() })) <= *dist_out,
                _ => false
        }),
        function: Box::new(move |data| Ok(data))
    })
}


fn make_base_laplace<NI: 'static, CI>(
    input_domain: DataDomain<NI, CI>,
    input_metric: DataMetric,
    output_measure: PrivacyMeasure,
    sigma: NI
) -> Result<Measurement<NI, CI>, Error>
    where NI: PartialOrd + Clone + Debug + Zero,
          CI: Eq + Clone + Debug,
          f64: From<NI> {

    if sigma < NI::zero() {
        return Err("Base Laplace: sigma must be greater than zero")
    }

    Ok(Measurement {
        input_domain: input_domain.clone(),
        input_metric: input_metric.clone(),
        output_measure: output_measure.clone(),
        privacy_relation: Box::new(move |dist_in, dist_out| match (dist_in, dist_out) {
            (DataDistance::DistFloat(sens), PrivacyDistance::PureDP(eps)) =>
                sens / f64::from(sigma.clone()) <= *eps,
            _ => false
        }),
        function: Box::new(move |data| Ok(data))
    })
}


fn make_noisy_sum<NI: 'static, CI: 'static>(
    input_domain: DataDomain<NI, CI>,
    input_metric: DataMetric,
    lower: NI, upper: NI, sigma: NI
) -> Result<Measurement<NI, CI>, Error>
    where NI: PartialOrd + Clone + Debug + Zero + Signed,
          CI: Eq + Clone + Debug,
          f64: From<NI> {

    let clamp_numeric = make_clamp_numeric(input_domain, input_metric, lower, upper)?;

    let sum = make_sum(
        clamp_numeric.input_domain.clone(),
        clamp_numeric.output_metric.clone(),
    )?;

    let base_laplace = make_base_laplace(
        sum.input_domain.clone(),
        sum.output_metric.clone(),
        PrivacyMeasure::ApproxDP(ApproxDP {}),
        sigma)?;

    base_laplace << (sum << clamp_numeric)?
    // make_mt_chain(base_laplace, make_tt_chain(sum, clamp_numeric, None)?, None)
}

