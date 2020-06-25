use std::cmp::Ordering;
use rand::Rng;

pub trait PartialMin: PartialOrd + Sized { fn partial_min(&self, other: &Self) -> Result<Self, &'static str>; }

pub trait PartialMax: PartialOrd + Sized { fn partial_max(&self, other: &Self) -> Result<Self, &'static str>; }

macro_rules! impl_float_partial {
    ($($source:ty),+) => {
        $(
            impl PartialMin for $source {
                fn partial_min(&self, other: &Self) -> Result<Self, &'static str> {
                    Ok(match self.partial_cmp(other) {
                        None => return Err("types may not be null when comparing"),
                        Some(Ordering::Less) => *self,
                        _ => *other
                    })
                }
            }

            impl PartialMax for $source {
                fn partial_max(&self, other: &Self) -> Result<Self, &'static str> {
                    Ok(match self.partial_cmp(other) {
                        None => return Err("types may not be null when comparing"),
                        Some(Ordering::Greater) => *self,
                        _ => *other
                    })
                }
            }
        )+
    }
}


macro_rules! impl_int_partial {
    ($($source:ty),+) => {
        $(
            impl PartialMin for $source {
                fn partial_min(&self, other: &Self) -> Result<Self, &'static str> { Ok(*self.min(other)) }
            }

            impl PartialMax for $source {
                fn partial_max(&self, other: &Self) -> Result<Self, &'static str> { Ok(*self.max(other)) }
            }
        )+
    }
}

impl_float_partial!(f64, f32);
impl_int_partial!(u8, u16, u32, u64, u128);


pub trait GenUniform: Sized {
    fn sample_uniform(min: Self, max: Self) -> Result<Self, &'static str>;
}

pub trait GenLaplace: GenUniform {
    fn sample_laplace(shift: Self, scale: Self) -> Result<Self, &'static str>;
}


macro_rules! impl_float_random {
    ($($source:ty),+) => {
        $(
            // No attempt has been made to make this noise remotely secure.
            impl GenUniform for $source {
                fn sample_uniform(min: Self, max: Self) -> Result<Self, &'static str> {
                    let mut rng = rand::thread_rng();
                    Ok(rng.gen_range(0.0, 1.0) as $source * (max - min) + min)
                }
            }

            impl GenLaplace for $source {
                fn sample_laplace(shift: Self, scale: Self) -> Result<Self, &'static str> {
                    let sample = Self::sample_uniform(0., 1.)?;
                    Ok(shift - scale * (sample - 0.5).signum() * (1. - 2. * (sample - 0.5).abs()).ln())
                }
            }
        )+
    }
}

impl_float_random!(f64, f32);



macro_rules! impl_int_random {
    ($($source:ty),+) => {
        $(
            // No attempt has been made to make this noise remotely secure.
            impl GenUniform for $source {
                fn sample_uniform(min: Self, max: Self) -> Result<Self, &'static str> {
                    let mut rng = rand::thread_rng();
                    Ok((rng.gen_range(0.0, 1.0) * (max - min) as f64) as $source + min)
                }
            }
        )+
    }
}

impl_int_random!(u8, u16, u32, u64, u128);

pub trait IsNull {
    fn is_null(&self) -> bool;
}

impl IsNull for f64 {
    fn is_null(&self) -> bool {
        !self.is_finite()
    }
}