use noisy_float::types::r64;
use opendp;
use opendp::{Error, Measurement};
use opendp::base::Data;
use opendp::base::domain::{Domain, Interval, Nature, ScalarDomain, VectorDomain};
use opendp::base::metric::{DataDistance, PrivacyDistance};
use opendp::base::value::*;
use opendp::*;
use opendp::constructors::aggregate::{make_sum, sensitivity_symmetric};
use opendp::constructors::chain::make_mt_chain;
use opendp::constructors::mechanisms::make_base_gaussian;
use opendp::Error::PrivacyMismatch;
use noisy_float::prelude::Float;


fn test_measurement(meas: Measurement, data: Data, in_dist: DataDistance) -> Result<(), Error> {

    let epsilon = Scalar::from(r64(1.)).to_numeric()?;
    let delta = Scalar::from(r64(0.0001)).to_numeric()?;
    let out_dist = PrivacyDistance::Approximate(epsilon, delta);
    let is_ok = meas.privacy_relation(&in_dist, &out_dist)?;

    println!("is_ok: {}", is_ok);

    let release = meas.function(data, &in_dist, &out_dist)?;

    if let Data::Value(Value::Scalar(v)) = release {
        println!("release: {:?}", v);
    };
    Ok(())
}

fn example_base_gauss() -> Result<(), Error> {

    let lower = Scalar::from(r64(0.0)).to_numeric()?;
    let upper = Scalar::from(r64(1000.0)).to_numeric()?;

    let original_domain = Domain::Scalar(ScalarDomain {
        may_have_nullity: false,
        nature: Nature::Numeric(Interval::new(Some(lower), Some(upper))?)
    });

    let base_gauss = make_base_gaussian(original_domain.clone(), r64(20.0).into())?;
    let data = Data::Value(Value::Scalar(r64(2.).into()));
    let in_dist = DataDistance::L2Sensitivity(Scalar::from(r64(1.)).to_numeric()?);

    test_measurement(base_gauss, data, in_dist);

    Ok(())
}

fn example_sum() -> Result<(), Error> {
    let lower: NumericScalar = r64(0.0).into();
    let upper: NumericScalar = r64(1.0).into();
    let original_domain = Domain::Vector(VectorDomain {
        length: Some(1000),
        is_nonempty: false,
        atomic_type: Box::new(Domain::Scalar(ScalarDomain {
            may_have_nullity: false,
            nature: Nature::Numeric(Interval::new(Some(lower.clone()), Some(upper.clone()))?)
        })),
    });

    let sum = make_sum(original_domain.clone())?;

    let base_gauss = make_base_gaussian(sum.output_domain(), r64(10.0).into())?;

    let measurement = make_mt_chain(
        base_gauss, sum,
        // NOTE: neither in distance or out distance are used
        Some(Box::new(move |_in_dist: &DataDistance, _out_dist: &PrivacyDistance| {
            let scalar: Scalar = apply_numeric_scalar!(sensitivity_symmetric, lower.clone(), upper.clone())?;
            Ok(DataDistance::L2Sensitivity(scalar.to_numeric()?))
        })))?;

    let raw_data = (0..1000).map(|_| r64(1.)).collect::<Vec<_>>();
    let data = Data::Value(Value::Vector(raw_data.into()));
    let in_dist = DataDistance::Hamming(1);

    test_measurement(measurement, data, in_dist)?;

    Ok(())
}

fn main() {
    example_base_gauss().unwrap();
    example_sum().unwrap();
}
