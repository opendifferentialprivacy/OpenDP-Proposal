// issue: parameters needs to be extremely generic, this doesn't cover bit depths
// issue: accessor patterns around parameters are verbose because of genericity
// issue: combinatorial types

use serde_json::Value;

struct AtomicDomain<T> {
    name: String,
    parameters: Value,
    valid_item: Box<dyn Fn(T) -> bool>,
    default_item: T
}

impl AtomicDomain<f64> {
    fn valid_item(self, x: f64) -> bool {

        let within_bounds = ||
            self.parameters.get("lower").and_then(|v| v.as_f64()) < Some(x)
                && Some(x) < self.parameters.get("upper").and_then(|v| v.as_f64());

        match self.name.to_str() {
            "Float" => true,
            "BoundedFloat" => within_bounds(),
            "OptionalBoundedFloat" => within_bounds(),
            "OptionalFloat" => true,
            _ => false
        }
    }

    fn default_item(self) -> f64 {
        match self.name.to_str() {
            "Float" => 0.,
            "BoundedFloat" => (
                self.parameters.get("lower").and_then(|v| v.as_f64()).unwrap() +
                    self.parameters.get("lower").and_then(|v| v.as_f64()).unwrap()) / 2,
            "OptionalBoundedFloat" => f64::NAN,
            "OptionalFloat" => f64::NAN,
            _ => f64::NAN
        }
    }
}

impl AtomicDomain<f64> {
    fn float() -> AtomicDomain<f64> {
        AtomicDomain::<f64> {
            name: "Float".to_string(),
            parameters: serde_json::Value::default(),
            valid_item: Box::new(move |v| true),
            default_item: 0.0
        }
    }

    fn bounded_float() -> AtomicDomain<f64> {
        AtomicDomain::<f64> {
            name: "Float".to_string(),
            parameters: serde_json::Value::default(),
            valid_item: Box::new(move |v| true),
            default_item: 0.0
        }
    }
}