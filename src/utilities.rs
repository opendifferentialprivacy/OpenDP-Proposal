// No attempt has been made to make this noise remotely secure.
use rand::Rng;

pub fn sample_standard_uniform() -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0.0, 1.0)
}

pub(crate) fn sample_laplace(scale: f64) -> f64 {
    let sample = sample_standard_uniform();
    let shift = 0.;
    shift - scale * (sample - 0.5).signum() * (1. - 2. * (sample - 0.5).abs()).ln()
}