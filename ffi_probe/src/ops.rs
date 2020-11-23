use std::any::TypeId;
use std::collections::HashMap;
use std::fmt::Debug;
use std::iter;
use std::str::FromStr;

use crate::core::Operation;
use crate::data::{Data, Form};

pub fn make_identity() -> Operation {
    let function = |arg: &Data| -> Data {
        arg.clone()
    };
    Operation::new(Box::new(function))
}

fn vec_string_to_str<'a>(src: &'a Vec<String>) -> Vec<&'a str> {
    src.into_iter().map(|e| e.as_str()).collect()
}

fn vec_str_to_string(src: Vec<&str>) -> Vec<String> {
    src.into_iter().map(|e| e.to_owned()).collect()
}

fn vec_vec_string_to_str<'a>(ret: &'a Vec<Vec<String>>) -> Vec<Vec<&'a str>> {
    ret.into_iter().map(vec_string_to_str).collect()
}

fn vec_vec_str_to_string(ret: Vec<Vec<&str>>) -> Vec<Vec<String>> {
    ret.into_iter().map(vec_str_to_string).collect()
}

fn split_lines(s: &str) -> Vec<&str> {
    s.lines().collect()
}

pub fn make_split_lines() -> Operation {
    let function = |arg: &Data| -> Data {
        let form: &String = arg.as_form();
        let lines = vec_str_to_string(split_lines(form));
        Data::new(lines)
    };
    Operation::new(Box::new(function))
}

fn split_records<'a>(separator: Option<&str>, lines: &Vec<&'a str>) -> Vec<Vec<&'a str>> {
    let separator = separator.unwrap_or(",");
    let mut cols: Vec<Vec<&str>> = Vec::new();
    for line in lines.into_iter() {
        let mut record: Vec<&str> = line.split(separator).into_iter().map(|e| e.trim()).collect();
        let delta = record.len() as isize - cols.len() as isize;
        if delta > 0 {
            let ac = if !cols.is_empty() { cols[0].len() } else { 0 };
            for _ in 0..delta {
                let col = vec![""; ac];
                cols.push(col);
            }
        } else {
            record.extend(iter::repeat("").take(-delta as usize));
        }
        for (i, val) in record.into_iter().enumerate() {
            cols[i].push(val);
        }
    }
    cols
}

pub fn make_split_records(separator: Option<&str>) -> Operation {
    let separator = separator.map(ToOwned::to_owned);
    let function = move |arg: &Data| -> Data {
        let form: &Vec<String> = arg.as_form();
        let form = vec_string_to_str(form);
        let ret = split_records(separator.as_deref(), &form);
        let ret: Vec<Vec<String>> = vec_vec_str_to_string(ret);
        Data::new(ret)
    };
    Operation::new(Box::new(function))
}

fn parse_col<T>(col: &Vec<&str>, default_on_error: bool) -> Vec<T> where
    T: FromStr + Default,
    T::Err: Debug {
    if default_on_error {
        col.into_iter().map(|e| e.parse().unwrap_or_else(|_| T::default())).collect()
    } else {
        col.into_iter().map(|e| e.parse().unwrap()).collect()
    }
}

pub fn make_parse_col<T>(default_on_error: bool) -> Operation where
    T: 'static + Form + Clone + PartialEq + FromStr + Default, T::Err: Debug {
    let function = move |arg: &Data| -> Data {
        let form: &Vec<String> = arg.as_form();
        let form = vec_string_to_str(form);
        let ret: Vec<T> = parse_col(&form, default_on_error);
        Data::new(ret)
    };
    Operation::new(Box::new(function))
}

pub type DataFrame = HashMap<String, Data>;

pub fn create_dataframe(cols: Vec<Vec<&str>>) -> DataFrame {
    cols.into_iter().enumerate().map(|(k, v)| (k.to_string(), vec_str_to_string(v).into())).collect()
}

pub fn make_create_dataframe() -> Operation {
    let function = |arg: &Data| -> Data {
        let form: &Vec<Vec<String>> = arg.as_form();
        let form = vec_vec_string_to_str(form);
        let ret = create_dataframe(form);
        Data::new(ret)
    };
    Operation::new(Box::new(function))
}

pub fn replace_col(key: &str, df: &DataFrame, col: &Data) -> DataFrame {
    let mut df = df.clone();
    *df.get_mut(key).unwrap() = col.clone();
    df
}

pub fn make_replace_col(key: String) -> Operation {
    let function = move |arg: &Data| -> Data {
        let form: &(DataFrame, Data) = arg.as_form();
        let ret = replace_col(&key, &form.0, &form.1);
        Data::new(ret)
    };
    Operation::new(Box::new(function))
}

pub fn parse_dataframe<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9>(separator: Option<&str>, impute: bool, s: &str) -> DataFrame where
    T0: 'static + Form + Clone + PartialEq + FromStr + Default, T0::Err: Debug,
    T1: 'static + Form + Clone + PartialEq + FromStr + Default, T1::Err: Debug,
    T2: 'static + Form + Clone + PartialEq + FromStr + Default, T2::Err: Debug,
    T3: 'static + Form + Clone + PartialEq + FromStr + Default, T3::Err: Debug,
    T4: 'static + Form + Clone + PartialEq + FromStr + Default, T4::Err: Debug,
    T5: 'static + Form + Clone + PartialEq + FromStr + Default, T5::Err: Debug,
    T6: 'static + Form + Clone + PartialEq + FromStr + Default, T6::Err: Debug,
    T7: 'static + Form + Clone + PartialEq + FromStr + Default, T7::Err: Debug,
    T8: 'static + Form + Clone + PartialEq + FromStr + Default, T8::Err: Debug,
    T9: 'static + Form + Clone + PartialEq + FromStr + Default, T9::Err: Debug {
    let lines = split_lines(s);
    let cols = split_records(separator, &lines);
    let df = create_dataframe(cols);
    fn parse_and_replace<T>(df: &DataFrame, impute: bool, key: &str) -> DataFrame where
        T: 'static + Form + Clone + PartialEq + FromStr + Default, T::Err: Debug {
        let type_id = TypeId::of::<T>();
        if type_id != TypeId::of::<()>() && type_id != TypeId::of::<String>() {
            let col = df.get(key).unwrap();
            let col = col.as_form();
            let col = vec_string_to_str(col);
            let col = parse_col::<T>(&col, impute);
            replace_col(key, df, &col.into())
        } else {
            df.clone()
        }
    }
    let df = parse_and_replace::<T0>(&df, impute, "0");
    let df = parse_and_replace::<T1>(&df, impute, "1");
    let df = parse_and_replace::<T2>(&df, impute, "2");
    let df = parse_and_replace::<T3>(&df, impute, "3");
    let df = parse_and_replace::<T4>(&df, impute, "4");
    let df = parse_and_replace::<T5>(&df, impute, "5");
    let df = parse_and_replace::<T6>(&df, impute, "6");
    let df = parse_and_replace::<T7>(&df, impute, "7");
    let df = parse_and_replace::<T8>(&df, impute, "8");
    let df = parse_and_replace::<T9>(&df, impute, "9");
    df
}

pub fn make_parse_dataframe<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9>(separator: Option<&str>, impute: bool) -> Operation where
    T0: 'static + Form + Clone + PartialEq + FromStr + Default, T0::Err: Debug,
    T1: 'static + Form + Clone + PartialEq + FromStr + Default, T1::Err: Debug,
    T2: 'static + Form + Clone + PartialEq + FromStr + Default, T2::Err: Debug,
    T3: 'static + Form + Clone + PartialEq + FromStr + Default, T3::Err: Debug,
    T4: 'static + Form + Clone + PartialEq + FromStr + Default, T4::Err: Debug,
    T5: 'static + Form + Clone + PartialEq + FromStr + Default, T5::Err: Debug,
    T6: 'static + Form + Clone + PartialEq + FromStr + Default, T6::Err: Debug,
    T7: 'static + Form + Clone + PartialEq + FromStr + Default, T7::Err: Debug,
    T8: 'static + Form + Clone + PartialEq + FromStr + Default, T8::Err: Debug,
    T9: 'static + Form + Clone + PartialEq + FromStr + Default, T9::Err: Debug {
    let separator = separator.map(ToOwned::to_owned);
    let function = move |arg: &Data| -> Data {
        let form: &String = arg.as_form();
        let ret = parse_dataframe::<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9>(separator.as_deref(), impute, &form);
        Data::new(ret)
    };
    Operation::new(Box::new(function))
}


mod ffi {
    use std::os::raw::c_char;

    use crate::ffi_utils;
    use crate::ffi_utils::c_bool;

    use super::*;

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_identity() -> *mut Operation {
        let operation = make_identity();
        ffi_utils::into_raw(operation)
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_split_lines() -> *mut Operation {
        let operation = make_split_lines();
        ffi_utils::into_raw(operation)
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_split_records(separator: *const c_char) -> *mut Operation {
        // TODO: Handle NULL for Option.
        let separator = Some(ffi_utils::to_str(separator));
        let operation = make_split_records(separator);
        ffi_utils::into_raw(operation)
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_parse_dataframe(separator: *const c_char, impute: c_bool) -> *mut Operation {
        // TODO: Handle NULL for Option.
        let separator = Some(ffi_utils::to_str(separator));
        let impute = ffi_utils::as_bool(impute);
        // FIXME: Hardwired generics.
        let operation = make_parse_dataframe::<String, i32, f64, String, String, String, String, String, String, String>(separator, impute);
        ffi_utils::into_raw(operation)
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__bootstrap() -> *const c_char {
        let spec =
r#"{
    "functions": [
        { "name": "make_identity", "ret": "void *" },
        { "name": "make_split_lines", "ret": "void *" },
        { "name": "make_split_records", "args": [ ["const char *", "separator"] ], "ret": "void *" },
        { "name": "make_parse_dataframe", "args": [ ["const char *", "separator"], ["bool", "impute"] ], "ret": "void *" }
    ]
}"#;
        ffi_utils::bootstrap(spec)
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_split_lines() {
        let operation = make_split_lines();
        let arg = "ant\nbat\ncat\n".to_owned();
        let arg = Data::new(arg);
        let ret = operation.invoke(&arg);
        let ret: Vec<String> = ret.into_form();
        assert_eq!(ret, vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()]);
    }

    #[test]
    fn test_make_split_records() {
        let operation = make_split_records(None);
        let arg = vec!["ant, foo".to_owned(), "bat, bar".to_owned(), "cat, baz".to_owned()];
        let arg = Data::new(arg);
        let ret = operation.invoke(&arg);
        let ret: Vec<Vec<String>> = ret.into_form();
        assert_eq!(ret, vec![
            vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()],
            vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()],
        ]);
    }

    #[test]
    fn test_make_create_dataframe() {
        let operation = make_create_dataframe();
        let arg = vec![
            vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()],
            vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()],
        ];
        let arg = Data::new(arg);
        let ret = operation.invoke(&arg);
        let ret: DataFrame = ret.into_form();
        let expected: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()])),
        ].into_iter().collect();
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_parse_col() {
        let operation = make_parse_col::<i32>(true);
        let arg = vec!["1".to_owned(), "2".to_owned(), "3".to_owned(), "foo".to_owned()];
        let arg = Data::new(arg);
        let ret = operation.invoke(&arg);
        let ret: Vec<i32> = ret.into_form();
        let expected = vec![1, 2, 3, 0];
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_parse_dataframe() {
        let operation = make_parse_dataframe::<String, i32, f64, String, String, String, String, String, String, String>(None, false);
        let arg = "ant, 1, 1.1\nbat, 2, 2.2\ncat, 3, 3.3".to_owned();
        let arg = Data::new(arg);
        let ret = operation.invoke(&arg);
        let ret: DataFrame = ret.into_form();
        let expected: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec![1, 2, 3])),
            ("2".to_owned(), Data::new(vec![1.1, 2.2, 3.3])),
        ].into_iter().collect();
        assert_eq!(ret, expected);
    }

}
