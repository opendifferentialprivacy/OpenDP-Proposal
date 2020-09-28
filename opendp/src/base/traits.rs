use noisy_float::types::{R32, R64};
use num::Float;

macro_rules! impl_trait_through_method {
    ($name:ident, $new_method:ident, $old_method:ident, $( $typ:ty),*) => {
        $(
            impl $name for $typ {
                fn $new_method(self) -> Self {
                    self.$old_method()
                }
            }
        )*
    }
}
/// Unified trait for computing the absolute value
/// .abs() is inconsistent across types.
/// Implementations are scattered across methods, Abs, Float, or missing
pub trait OmniAbs {
    fn omni_abs(self) -> Self;
}
impl_trait_through_method!(OmniAbs, omni_abs, abs, f32, f64, R32, R64, i8, i16, i32, i64, i128);
macro_rules! impl_unsigned_abs {
    ($($typ:ty),*) => ($(impl OmniAbs for $typ {fn omni_abs(self) -> Self {self}})*)
}
impl_unsigned_abs!(u8, u16, u32, u64, u128);