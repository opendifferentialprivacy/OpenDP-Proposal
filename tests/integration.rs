// use opendp_proposal::components::{make_noisy_sum, make_clamp, make_mt_chain, make_imputation, make_tt_chain};
// use opendp_proposal::{Domain, Bounds};

// #[test]
// fn analysis_sum() -> Result<(), &'static str> {
//
//     let input_properties = Domain {
//         has_nullity: true,
//         bounds: None
//     };
//
//
//     let imputer = make_imputation(input_properties, 0.5, 1.5)?;
//     let clamper = make_clamp(imputer.output_domain.clone(), 0.5, 1.5)?;
//
//     let preprocessed = make_tt_chain(clamper, imputer)?;
//
//     let noisy_sum = make_noisy_sum(preprocessed.output_domain.clone(), 0.5)?;
//     let chained = make_mt_chain(noisy_sum, preprocessed)?;
//
//     let num_records = 100;
//     let data = (0..num_records).map(|_| 1f64).collect();
//
//     let sum = (chained.function)(data)?;
//
//     println!("noised sum: {:?}", sum);
//
//     Ok(())
// }

// #[test]
// fn analysis_sum_fail() -> Result<(), &'static str> {
//     let clamper = make_clamp(0.5f32, 1.5)?;
//     let noisy_sum = make_noisy_sum(0.5f32, 1.5, 0.5)?;
//     let chained = make_mt_chain(noisy_sum, clamper)?;
//
//     Ok(())
// }