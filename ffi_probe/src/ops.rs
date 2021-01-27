use std::collections::HashMap;
use std::fmt::Debug;
use std::iter;
use std::iter::Sum;
use std::ops::Bound;
use std::str::FromStr;

use rand::Rng;

use crate::core::{Measurement, Transformation};
use crate::data::{Data, Element, Form};
use crate::dom::{AllDomain, IntervalDomain, MapDomain, VectorDomain};

pub fn make_identity<T: Clone>() -> Transformation<AllDomain<T>, AllDomain<T>> {
    let input_domain = AllDomain::<T>::new();
    let output_domain = AllDomain::<T>::new();
    let function = |arg: &T| -> T {
        arg.clone()
    };
    Transformation::new(input_domain, output_domain, function)
}

fn vec_string_to_str(src: &Vec<String>) -> Vec<&str> {
    src.into_iter().map(|e| e.as_str()).collect()
}

fn vec_str_to_string(src: Vec<&str>) -> Vec<String> {
    src.into_iter().map(|e| e.to_owned()).collect()
}

fn split_lines(s: &str) -> Vec<&str> {
    s.lines().collect()
}

pub fn make_split_lines() -> Transformation<AllDomain<String>, VectorDomain<AllDomain<String>>> {
    let input_domain = AllDomain::<String>::new();
    let output_domain = VectorDomain::new_all();
    let function = |arg: &String| -> Vec<String> {
        let ret = split_lines(arg);
        vec_str_to_string(ret)
    };
    Transformation::new(input_domain, output_domain, function)
}

fn parse_series<T>(col: &Vec<&str>, default_on_error: bool) -> Vec<T> where
    T: FromStr + Default,
    T::Err: Debug {
    if default_on_error {
        col.into_iter().map(|e| e.parse().unwrap_or_else(|_| T::default())).collect()
    } else {
        col.into_iter().map(|e| e.parse().unwrap()).collect()
    }
}

pub fn make_parse_series<T>(impute: bool) -> Transformation<VectorDomain<AllDomain<String>>, VectorDomain<AllDomain<T>>> where
    T: FromStr + Default, T::Err: Debug {
    let input_domain = VectorDomain::new_all();
    let output_domain = VectorDomain::new_all();
    let function = move |arg: &Vec<String>| -> Vec<T> {
        let arg = vec_string_to_str(arg);
        parse_series(&arg, impute)
    };
    Transformation::new(input_domain, output_domain, function)
}

fn split_records<'a>(separator: &str, lines: &Vec<&'a str>) -> Vec<Vec<&'a str>> {
    fn split<'a>(line: &'a str, separator: &str) -> Vec<&'a str> {
        line.split(separator).into_iter().map(|e| e.trim()).collect()
    }
    lines.into_iter().map(|e| split(e, separator)).collect()
}

pub fn make_split_records(separator: Option<&str>) -> Transformation<VectorDomain<AllDomain<String>>, VectorDomain<VectorDomain<AllDomain<String>>>> {
    let separator = separator.unwrap_or(",").to_owned();
    let input_domain = VectorDomain::new_all();
    let output_domain = VectorDomain::new(VectorDomain::new_all());
    let function = move |arg: &Vec<String>| -> Vec<Vec<String>> {
        let arg = vec_string_to_str(arg);
        let ret = split_records(&separator, &arg);
        ret.into_iter().map(vec_str_to_string).collect()
    };
    Transformation::new(input_domain, output_domain, function)
}

fn conform_records<'a>(len: usize, records: &Vec<Vec<&'a str>>) -> Vec<Vec<&'a str>> {
    fn conform<'a>(record: &Vec<&'a str>, len: usize) -> Vec<&'a str> {
        if record.len() > len {
            record[0..len].to_vec().clone()
        } else if record.len() < len {
            record.clone().into_iter().chain(iter::repeat("").take(len - record.len())).collect()
        } else {
            record.clone()
        }
    }
    records.into_iter().map(|e| conform(e, len)).collect()
}

pub type DataFrame = HashMap<String, Data>;

fn create_dataframe(col_count: usize, records: &Vec<Vec<&str>>) -> DataFrame {
    let records = conform_records(col_count, &records);
    let mut cols = vec![Vec::new(); col_count];
    for record in records.into_iter() {
        for i in 0..col_count {
            cols[i].push(record[i])
        }
    }
    cols.into_iter().enumerate().map(|(k, v)| (k.to_string(), Data::new(vec_str_to_string(v)))).collect()
}

pub fn create_dataframe_domain() -> MapDomain<AllDomain<Data>> {
    MapDomain::new(AllDomain::new())
}

pub fn make_create_dataframe(col_count: usize) -> Transformation<VectorDomain<VectorDomain<AllDomain<String>>>, MapDomain<AllDomain<Data>>> {
    let input_domain = VectorDomain::new(VectorDomain::new_all());
    let output_domain = create_dataframe_domain();
    let function = move |arg: &Vec<Vec<String>>| -> DataFrame {
        let arg = arg.into_iter().map(|e| vec_string_to_str(e)).collect();
        create_dataframe(col_count, &arg)
    };
    Transformation::new(input_domain, output_domain, function)
}

fn split_dataframe<'a>(separator: &str, col_count: usize, s: &str) -> DataFrame {
    let lines = split_lines(s);
    let records = split_records(separator, &lines);
    let records = conform_records(col_count, &records);
    create_dataframe(col_count, &records)
}

pub fn make_split_dataframe(separator: Option<&str>, col_count: usize) -> Transformation<AllDomain<String>, MapDomain<AllDomain<Data>>> {
    let separator = separator.unwrap_or(",").to_owned();
    let input_domain = AllDomain::new();
    let output_domain = create_dataframe_domain();
    let function = move |arg: &String| -> DataFrame {
        split_dataframe(&separator, col_count, &arg)
    };
    Transformation::new(input_domain, output_domain, function)
}

fn replace_col(key: &str, df: &DataFrame, col: &Data) -> DataFrame {
    let mut df = df.clone();
    *df.get_mut(key).unwrap() = col.clone();
    df
}

fn parse_column<T>(key: &str, impute: bool, df: &DataFrame) -> DataFrame where
    T: 'static + Element + Clone + PartialEq + FromStr + Default, T::Err: Debug {
    let col = df.get(key).unwrap();
    let col = col.as_form();
    let col = vec_string_to_str(col);
    let col = parse_series::<T>(&col, impute);
    replace_col(key, &df, &col.into())
}

pub fn make_parse_column<T>(key: &str, impute: bool) -> Transformation<MapDomain<AllDomain<Data>>, MapDomain<AllDomain<Data>>> where
    T: 'static + Element + Clone + PartialEq + FromStr + Default, T::Err: Debug {
    let key = key.to_owned();
    let input_domain = create_dataframe_domain();
    let output_domain = create_dataframe_domain();
    let function = move |arg: &DataFrame| -> DataFrame {
        parse_column::<T>(&key, impute, arg)
    };
    Transformation::new(input_domain, output_domain, function)
}

pub fn make_select_column<T>(key: &str) -> Transformation<MapDomain<AllDomain<Data>>, VectorDomain<AllDomain<T>>> where
    T: 'static + Element + Clone + PartialEq {
    let key = key.to_owned();
    let input_domain = create_dataframe_domain();
    let output_domain = VectorDomain::new_all();
    let function = move |arg: &DataFrame| -> Vec<T> {
        let ret = arg.get(&key).expect("Missing dataframe column");
        let ret: &Vec<T> = ret.as_form();
        ret.clone()
    };
    Transformation::new(input_domain, output_domain, function)
}

fn clamp<T: Copy + PartialOrd>(lower: T, upper: T, x: &Vec<T>) -> Vec<T> {
    fn clamp1<T: Copy + PartialOrd>(lower: T, upper: T, x: T) -> T {
        if x < lower { lower } else if x > upper { upper } else { x }
    }
    x.into_iter().map(|e| clamp1(lower, upper, *e)).collect()
}

pub fn make_clamp<T>(lower: T, upper: T) -> Transformation<VectorDomain<AllDomain<T>>, VectorDomain<IntervalDomain<T>>> where
    T: 'static + Copy + PartialOrd {
    let input_domain = VectorDomain::new_all();
    let output_domain = VectorDomain::new(IntervalDomain::new(Bound::Included(lower), Bound::Included(upper)));
    let function = move |arg: &Vec<T>| -> Vec<T> {
        clamp(lower, upper, arg)
    };
    Transformation::new(input_domain, output_domain, function)
}

pub fn make_bounded_sum<T>(lower: T, upper: T) -> Transformation<VectorDomain<IntervalDomain<T>>, AllDomain<T>> where
    T: 'static + Clone + PartialOrd + Sum<T> {
    let input_domain = VectorDomain::new(IntervalDomain::new(Bound::Included(lower), Bound::Included(upper)));
    let output_domain = AllDomain::new();
    let function = |arg: &Vec<T>| -> T {
        // FIXME: Can't make this work with references, have to clone.
        let arg = arg.clone();
        arg.into_iter().sum()
    };
    Transformation::new(input_domain, output_domain, function)
}

fn laplace(sigma: f64) -> f64 {
    let mut rng = rand::thread_rng();
    let u: f64 = rng.gen_range(-0.5, 0.5);
    u.signum() * (1.0 - 2.0 * u.abs()).ln() * sigma
}

pub trait AddNoise {
    fn add_noise(self, noise: f64) -> Self;
}
impl AddNoise for u32 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }
impl AddNoise for u64 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }
impl AddNoise for i32 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }
impl AddNoise for i64 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }
impl AddNoise for f32 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }
impl AddNoise for f64 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }
impl AddNoise for u8 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }

pub fn make_base_laplace<T>(sigma: f64) -> Measurement<AllDomain<T>, AllDomain<T>> where
    T: Copy + AddNoise {
    let input_domain = AllDomain::new();
    let output_domain = AllDomain::new();
    let function = move |arg: &T| -> T {
        let noise = laplace(sigma);
        arg.add_noise(noise)
    };
    Measurement::new(input_domain, output_domain, function)
}


mod ffi {
    use std::os::raw::{c_char, c_uint, c_void};

    use crate::core::ffi::{FfiMeasurement, FfiTransformation};
    use crate::ffi_utils;
    use crate::ffi_utils::c_bool;
    use crate::ffi_utils::TypeArgs;

    use super::*;

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_identity(type_args: *const c_char) -> *mut FfiTransformation {
        fn monomorphize<T: 'static + Form + Clone>() -> *mut FfiTransformation {
            let transformation = make_identity::<T>();
            FfiTransformation::new_from_types(transformation)
        }
        let type_args = TypeArgs::expect(type_args, 1);
        dispatch!(monomorphize, [(type_args.0[0], @primitives)], ())
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_split_lines() -> *mut FfiTransformation {
        let transformation = make_split_lines();
        FfiTransformation::new_from_types(transformation)
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_parse_series(type_args: *const c_char, impute: c_bool) -> *mut FfiTransformation {
        fn monomorphize<T>(impute: bool) -> *mut FfiTransformation where
            T: 'static + FromStr + Default, T::Err: Debug {
            let transformation = make_parse_series::<T>(impute);
            FfiTransformation::new_from_types(transformation)
        }
        let type_args = TypeArgs::expect(type_args, 1);
        let impute = ffi_utils::to_bool(impute);
        dispatch!(monomorphize, [(type_args.0[0], @primitives)], (impute))
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_split_records(separator: *const c_char) -> *mut FfiTransformation {
        let separator = ffi_utils::to_option_str(separator);
        let transformation = make_split_records(separator);
        FfiTransformation::new_from_types(transformation)
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_create_dataframe(col_count: c_uint) -> *mut FfiTransformation {
        let col_count = col_count as usize;
        let transformation = make_create_dataframe(col_count);
        FfiTransformation::new_from_types(transformation)
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_split_dataframe(separator: *const c_char, col_count: c_uint) -> *mut FfiTransformation {
        let separator = ffi_utils::to_option_str(separator);
        let col_count = col_count as usize;
        let transformation = make_split_dataframe(separator, col_count);
        FfiTransformation::new_from_types(transformation)
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_parse_column(type_args: *const c_char, key: *const c_char, impute: c_bool) -> *mut FfiTransformation {
        fn monomorphize<T>(key: &str, impute: bool) -> *mut FfiTransformation where
            T: 'static + Element + Clone + PartialEq + FromStr + Default, T::Err: Debug {
            let transformation = make_parse_column::<T>(key, impute);
            FfiTransformation::new_from_types(transformation)
        }
        let type_args = TypeArgs::expect(type_args, 1);
        let key = ffi_utils::to_str(key);
        let impute = ffi_utils::to_bool(impute);
        dispatch!(monomorphize, [(type_args.0[0], @primitives)], (key, impute))
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_select_column(type_args: *const c_char, key: *const c_char) -> *mut FfiTransformation {
        fn monomorphize<T>(key: &str) -> *mut FfiTransformation where
            T: 'static + Element + Clone + PartialEq {
            let transformation = make_select_column::<T>(key);
            FfiTransformation::new_from_types(transformation)
        }
        let type_args = TypeArgs::expect(type_args, 1);
        let key = ffi_utils::to_str(key);
        dispatch!(monomorphize, [(type_args.0[0], @primitives)], (key))
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_clamp(type_args: *const c_char, lower: *const c_void, upper: *const c_void) -> *mut FfiTransformation {
        fn monomorphize<T>(lower: *const c_void, upper: *const c_void) -> *mut FfiTransformation where
            T: 'static + Copy + PartialOrd {
            let lower = ffi_utils::as_ref(lower as *const T).clone();
            let upper = ffi_utils::as_ref(upper as *const T).clone();
            let transformation = make_clamp::<T>(lower, upper);
            FfiTransformation::new_from_types(transformation)
        }
        let type_args = TypeArgs::expect(type_args, 1);
        dispatch!(monomorphize, [(type_args.0[0], @numbers)], (lower, upper))
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_bounded_sum(type_args: *const c_char, lower: *const c_void, upper: *const c_void) -> *mut FfiTransformation {
        fn monomorphize<T>(lower: *const c_void, upper: *const c_void) -> *mut FfiTransformation where
            T: 'static + Clone + PartialOrd + Sum {
            let lower = ffi_utils::as_ref(lower as *const T).clone();
            let upper = ffi_utils::as_ref(upper as *const T).clone();
            let transformation = make_bounded_sum::<T>(lower, upper);
            FfiTransformation::new_from_types(transformation)
        }
        let type_args = TypeArgs::expect(type_args, 1);
        dispatch!(monomorphize, [(type_args.0[0], @numbers)], (lower, upper))
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_base_laplace(type_args: *const c_char, sigma: f64) -> *mut FfiMeasurement {
        fn monomorphize<T>(sigma: f64) -> *mut FfiMeasurement where
            T: 'static + Copy + PartialEq + AddNoise {
            let measurement = make_base_laplace::<T>(sigma);
            FfiMeasurement::new_from_types(measurement)
        }
        let type_args = TypeArgs::expect(type_args, 1);
        dispatch!(monomorphize, [(type_args.0[0], @numbers)], (sigma))
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__bootstrap() -> *const c_char {
        let spec =
r#"{
    "functions": [
        { "name": "make_identity", "ret": "void *" },
        { "name": "make_split_lines", "ret": "void *" },
        { "name": "make_parse_series", "args": [ ["const char *", "selector"], ["bool", "impute"] ], "ret": "void *" },
        { "name": "make_split_records", "args": [ ["const char *", "separator"] ], "ret": "void *" },
        { "name": "make_create_dataframe", "args": [ ["unsigned int", "col_count"] ], "ret": "void *" },
        { "name": "make_split_dataframe", "args": [ ["const char *", "separator"], ["unsigned int", "col_count"] ], "ret": "void *" },
        { "name": "make_parse_column", "args": [ ["const char *", "selector"], ["const char *", "key"], ["bool", "impute"] ], "ret": "void *" },
        { "name": "make_select_column", "args": [ ["const char *", "selector"], ["const char *", "key"] ], "ret": "void *" },
        { "name": "make_clamp", "args": [ ["const char *", "selector"], ["void *", "lower"], ["void *", "upper"] ], "ret": "void *" },
        { "name": "make_bounded_sum", "args": [ ["const char *", "selector"], ["void *", "lower"], ["void *", "upper"] ], "ret": "void *" },
        { "name": "make_base_laplace", "args": [ ["const char *", "selector"], ["double", "sigma"] ], "ret": "void *" }
    ]
}"#;
        ffi_utils::bootstrap(spec)
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::make_chain_tt;

    #[test]
    fn test_identity() {
        let identity = make_identity();
        let arg = 99;
        let ret = identity.function.eval(&arg);
        assert_eq!(ret, 99);
    }

    #[test]
    fn test_make_split_lines() {
        let transformation = make_split_lines();
        let arg = "ant\nbat\ncat\n".to_owned();
        let ret = transformation.function.eval(&arg);
        assert_eq!(ret, vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()]);
    }

    #[test]
    fn test_make_parse_series() {
        let transformation = make_parse_series::<i32>(true);
        let arg = vec!["1".to_owned(), "2".to_owned(), "3".to_owned(), "foo".to_owned()];
        let ret = transformation.function.eval(&arg);
        let expected = vec![1, 2, 3, 0];
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_split_records() {
        let transformation = make_split_records(None);
        let arg = vec!["ant, foo".to_owned(), "bat, bar".to_owned(), "cat, baz".to_owned()];
        let ret = transformation.function.eval(&arg);
        assert_eq!(ret, vec![
            vec!["ant".to_owned(), "foo".to_owned()],
            vec!["bat".to_owned(), "bar".to_owned()],
            vec!["cat".to_owned(), "baz".to_owned()],
        ]);
    }

    #[test]
    fn test_make_create_dataframe() {
        let transformation = make_create_dataframe(2);
        let arg = vec![
            vec!["ant".to_owned(), "foo".to_owned()],
            vec!["bat".to_owned(), "bar".to_owned()],
            vec!["cat".to_owned(), "baz".to_owned()],
        ];
        let ret = transformation.function.eval(&arg);
        let expected: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()])),
        ].into_iter().collect();
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_split_dataframe() {
        let transformation = make_split_dataframe(None, 2);
        let arg = "ant, foo\nbat, bar\ncat, baz".to_owned();
        let ret = transformation.function.eval(&arg);
        let expected: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()])),
        ].into_iter().collect();
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_parse_column() {
        let transformation = make_parse_column::<i32>("1", true);
        let arg: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec!["1".to_owned(), "2".to_owned(), "".to_owned()])),
        ].into_iter().collect();
        let ret = transformation.function.eval(&arg);
        let expected: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec![1, 2, 0])),
        ].into_iter().collect();
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_parse_columns() {
        let transformation0 = make_parse_column::<i32>("1", true);
        let transformation1 = make_parse_column::<f64>("2", true);
        let transformation = make_chain_tt(&transformation1, &transformation0);
        let arg: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec!["1".to_owned(), "2".to_owned(), "3".to_owned()])),
            ("2".to_owned(), Data::new(vec!["1.1".to_owned(), "2.2".to_owned(), "3.3".to_owned()])),
        ].into_iter().collect();
        let ret = transformation.function.eval(&arg);
        let expected: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec![1, 2, 3])),
            ("2".to_owned(), Data::new(vec![1.1, 2.2, 3.3])),
        ].into_iter().collect();
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_select_column() {
        let transformation = make_select_column::<String>("1");
        let arg: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()])),
        ].into_iter().collect();
        let ret = transformation.function.eval(&arg);
        let expected = vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()];
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_clamp() {
        let transformation = make_clamp(0, 10);
        let arg = vec![-10, -5, 0, 5, 10, 20];
        let ret = transformation.function.eval(&arg);
        let expected = vec![0, 0, 0, 5, 10, 10];
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_bounded_sum() {
        let transformation = make_bounded_sum::<i32>(0, 10);
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg);
        let expected = 15;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_base_laplace() {
        let measurement = make_base_laplace::<f64>(1.0);
        let arg = 0.0;
        let _ret = measurement.function.eval(&arg);
        // TODO: Test for base_laplace
    }

}
