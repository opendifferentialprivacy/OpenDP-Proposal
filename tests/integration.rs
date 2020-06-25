use opendp_proposal::components::{make_noisy_sum, make_clamp, make_mt_chain};

#[test]
fn analysis_sum() -> Result<(), &'static str> {
    let clamper = make_clamp(0.5.into(), 1.5.into())?;
    let noisy_sum = make_noisy_sum(0.5.into(), 1.5.into(), 0.5)?;
    let chained = make_mt_chain(noisy_sum, clamper)?;

    let num_records = 100;
    let data = (0..num_records).map(|_| 1.).collect::<Vec<f64>>();

    let sum = (chained.function)(data)?;

    println!("noised sum: {:?}", sum);

    Ok(())
}