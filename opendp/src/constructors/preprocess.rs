
use crate::{Transformation, Error};
use crate::base::domain::{ScalarDomain, Interval, Nature, VectorDomain};
use crate::base::metric::{Metric, Distance};
// use crate::base::Data;
use crate::base::{functions as fun, Data};
use noisy_float::types::{R64, r64};

fn clamp<T: PartialOrd>(v: T, l: T, u: T) -> Result<T, Error> {
    fun::min(fun::max(v, l)?, u)
}

fn clamp_vec<T: PartialOrd + Clone>(v: Vec<T>, l: T, u: T) -> Result<Vec<T>, Error> {
    v.into_iter().map(|v| clamp(v, l.clone(), u.clone())).collect()
}

type ClampDomain<T> = VectorDomain<Vec<T>, ScalarDomain<T, T>>;
pub fn make_clamp_numeric<I, T, M>(
    input_domain: ClampDomain<T>, input_metric: Metric, lower: T, upper: T
) -> Result<Transformation<ClampDomain<T>, ClampDomain<T>, M, M>, Error>
    where T: 'static + PartialOrd + Clone,
          M: 'static + PartialOrd + Clone {

    let clamp_atomic_domain = |atomic_type: ScalarDomain<T, T>| -> Result<ScalarDomain<T, T>, Error> {
        let ScalarDomain { may_have_nullity, nature, container } = atomic_type;

        let (prior_lower, prior_upper) = nature.clone().to_numeric()?.bounds();

        let lower: T = prior_lower
            .map(|prior_lower| fun::max(&lower, &prior_lower).map(|v| v.clone()))
            .transpose()?.unwrap_or(lower.clone());

        let upper: T = prior_upper
            .map(|prior_upper| fun::min(&upper, &prior_upper).map(|v| v.clone()))
            .transpose()?.unwrap_or(upper.clone());

        Ok(ScalarDomain {
            may_have_nullity,
            nature: Nature::Numeric(Interval::new(Some(lower), Some(upper))?),
            container
        })
    };

    let mut output_domain = input_domain.clone();
    output_domain.atomic_type = clamp_atomic_domain(output_domain.atomic_type)?;

    Ok(Transformation {
        input_domain,
        input_metric: input_metric.clone(),
        output_domain,
        output_metric: input_metric.clone(),
        stability_relation: Box::new(move |in_dist: &Distance<M>, out_dist: &Distance<M>| Ok(in_dist <= out_dist)),
        function: Box::new(move |data: Data<Vec<T>>| match data {
            Data::Value(data) => {
                Ok(Data::Value(clamp_vec(data, lower.clone(), upper.clone())?))
            },
            _ => Err(Error::NotImplemented)
        })
    })
}

trait Impute {
    type Output;
    fn impute(self, lower: Self::Output, upper: Self::Output) -> Self::Output;
}

impl Impute for f64 {
    type Output = R64;
    fn impute(self, lower: R64, upper: R64) -> Self::Output {
        if self.is_finite() { r64(self) } else { (lower + upper) / r64(2.) }
    }
}

impl Impute for Option<i64> {
    type Output = i64;
    fn impute(self, lower: i64, upper: i64) -> Self::Output {
        self.unwrap_or_else(|| (lower + upper) / 2)
    }
}

// pub fn make_impute_integer<T: Impute<Output=U>, U: Clone + PartialOrd>(
//     input_domain: &dyn Domain<Vec<T>>, lower: U, upper: U,
// ) -> Result<Transformation<Vec<T>, Vec<U>>, Error> {
//     if lower > upper {
//         return Err(Error::Raw("lower may not be greater than upper".to_string()))
//     }
//
//     // function that applies impute transformation to atomic type
//     let impute_atomic_domain = |atomic_type: &dyn Domain<T>| -> Result<Box<dyn Domain<T>>, Error> {
//         let ScalarDomain { may_have_nullity, nature } = atomic_type
//             .as_any().downcast_ref::<ScalarDomain<T>>()
//             .ok_or(Error::InvalidDomain)?;
//
//         // retrieve lower and upper bounds for the data domain
//         let (prior_lower, prior_upper) = nature.to_numeric()?.bounds();
//
//         // if lower bound on the input domain exists, then potentially widen it or return none
//         let lower = Some(prior_lower
//             .map(|prior_lower| fun::max(&lower, &prior_lower))
//             .transpose()?.unwrap_or(lower.clone()));
//
//         // if upper bound on the input domain exists, then potentially widen it or return none
//         let upper = Some(prior_upper
//             .map(|prior_upper| fun::min(&upper, &prior_upper))
//             .transpose()?.unwrap_or(upper.clone()));
//
//         Ok(Box::new(ScalarDomain {
//             may_have_nullity: false,
//             nature: Nature::Numeric(Interval::new(lower, upper)?)
//         }))
//     };
//
//     let VectorDomain {
//         atomic_type, is_nonempty, length, container
//     } = input_domain.as_any().downcast_ref::<VectorDomain<Vec<T>, T>>()
//         .ok_or(Error::InvalidDomain)?;
//
//     Ok(Transformation {
//         input_domain: Box::new(input_domain),
//         output_domain: Box::new(VectorDomain {
//             atomic_type: impute_atomic_domain(atomic_type)?,
//             is_nonempty: is_nonempty.clone(),
//             length: length.clone(),
//             container: container.clone()
//         }),
//         stability_relation: Box::new(move |d_in: &DataDistance, d_out: &DataDistance| Ok(d_in <= d_out)),
//         function: Box::new(move |data: Data<T>| match data {
//             Data::Value(data) => Ok(Data::Value(impute_int_vec(data.to_vector()?,lower_wrap.clone(), upper_wrap.clone())?)),
//             _ => Err(Error::NotImplemented)
//         })
//     })
// }

// #[cfg(test)]
// pub mod test_impute_numeric {
//     use crate::constructors::preprocess::make_impute_integer;
//     use crate::base::domain::{Domain, VectorDomain};
//     use crate::base::value::Scalar;
//
//     #[test]
//     fn test_1() {
//         let input_domain = Domain::Vector(VectorDomain {
//             atomic_type: Box::new(Domain::numeric_scalar(None, None, true).unwrap()),
//             is_nonempty: false,
//             length: None,
//         });
//
//         make_impute_integer(
//             &input_domain,
//             2.into(),
//             10.into()).unwrap();
//
//         if !make_impute_integer(
//             &input_domain,
//             20.into(),
//             10.into()).is_err() {
//             panic!("Impute must fail if bounds are unordered.")
//         }
//     }
// }