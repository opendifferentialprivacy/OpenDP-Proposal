// issue: equality checking by value is not possible
// issue: combinatorial types

trait AtomicDomain<T> {
    fn name(self) -> String;
    fn valid_item(self, x: T) -> bool;
    fn default_item(self) -> T;
}


struct BoundedFloat<T> {
    lower: T,
    upper: T
}
impl AtomicDomain<f64> for BoundedFloat<f64> {
    fn name(self) -> String {
        "BoundedFloatf64".to_string()
    }

    fn valid_item(self, x: f64) -> bool {
        self.lower < x && x < self.upper
    }

    fn default_item(self) -> f64 {
        (self.lower + self.upper) / 2.
    }
}

struct Float<T> {}
impl AtomicDomain<f64> for BoundedFloat<f64> {
    fn name(self) -> String {
        "Floatf64".to_string()
    }

    fn valid_item(self, x: f64) -> bool {
        x.is_finite()
    }

    fn default_item(self) -> f64 {
        0.
    }
}
