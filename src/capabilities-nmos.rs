use serde_json::{json, Value};
use regex::Regex;
use serde::de::DeserializeOwned;


use crate::json_fields::{self, JsonField};

fn make_caps_string_constraint(enum_values: &[String], pattern: &str) -> Value {
    json!({
        json_fields::CONSTRAINT_ENUM.key: enum_values,
        json_fields::CONSTRAINT_PATTERN.key: pattern,
    })
}

fn make_caps_integer_constraint(enum_values: &[i64], minimum: i64, maximum: i64) -> Value {
    json!({
        json_fields::CONSTRAINT_ENUM.key: enum_values,
        json_fields::CONSTRAINT_MINIMUM.key: minimum,
        json_fields::CONSTRAINT_MAXIMUM.key: maximum,
    })
}

fn make_caps_number_constraint(enum_values: &[f64], minimum: f64, maximum: f64) -> Value {
    json!({
        json_fields::CONSTRAINT_ENUM.key: enum_values,
        json_fields::CONSTRAINT_MINIMUM.key: minimum,
        json_fields::CONSTRAINT_MAXIMUM.key: maximum,
    })
}

fn make_caps_boolean_constraint(enum_values: &[bool]) -> Value {
    json!({
        json_fields::CONSTRAINT_ENUM.key: enum_values,
    })
}

fn make_caps_rational_constraint(enum_values: &[crate::rational::Rational], minimum: crate::rational::Rational, maximum: crate::rational::Rational) -> Value {
    json!({
        json_fields::CONSTRAINT_ENUM.key: enum_values.iter().map(|r| crate::rational::make_rational(r)).collect::<Vec<Value>>(),
        json_fields::CONSTRAINT_MINIMUM.key: crate::rational::make_rational(minimum),
        json_fields::CONSTRAINT_MAXIMUM.key: crate::rational::make_rational(maximum),
    })
}

mod details {
    use serde_json::{Number, Value};

    use crate::json_fields::{self, JsonField};

    fn match_enum_constraint<T, F>(value: &T, constraint: &Value, parse: F) -> bool
    where
        F: Fn(&Value) -> Option<T>,
    {
        if let Some(enum_values) = constraint.get(json_fields::CONSTRAINT_ENUM.key) {
            if let Some(enum_values) = enum_values.as_array() {
                if !enum_values.iter().any(|enum_value| parse(enum_value) == Some(value)) {
                    return false;
                }
            }
        }
        true
    }
}



fn match_enum_constraint<T: DeserializeOwned>(value: T, constraint: &Value) -> bool {
    if constraint.has_key("enum") {
        let values = constraint["enum"].as_array().unwrap();
        return values.iter().any(|v| v == &serde_json::to_value(&value).unwrap());
    }
    true
}

fn match_minimum_maximum_constraint<T: PartialOrd + DeserializeOwned>(
    value: T,
    constraint: &Value,
) -> bool {
    if constraint.has_key("minimum") {
        let minimum: T = constraint["minimum"].as_str().unwrap().parse().unwrap();
        if minimum > value {
            return false;
        }
    }
    if constraint.has_key("maximum") {
        let maximum: T = constraint["maximum"].as_str().unwrap().parse().unwrap();
        if maximum < value {
            return false;
        }
    }
    true
}

fn match_pattern_constraint(value: &str, constraint: &Value) -> bool {
    if constraint.has_key("pattern") {
        let pattern = constraint["pattern"].as_str().unwrap();
        let regex = Regex::new(pattern).unwrap();
        if !regex.is_match(value) {
            return false;
        }
    }
    true
}

pub fn match_string_constraint(value: &str, constraint: &Value) -> bool {
    match_enum_constraint(value, constraint) && match_pattern_constraint(value, constraint)
}

pub fn match_integer_constraint(value: i64, constraint: &Value) -> bool {
    match_enum_constraint(value, constraint) && match_minimum_maximum_constraint(value, constraint)
}

pub fn match_number_constraint(value: f64, constraint: &Value) -> bool {
    match_enum_constraint(value, constraint) && match_minimum_maximum_constraint(value, constraint)
}

pub fn match_boolean_constraint(value: bool, constraint: &Value) -> bool {
    match_enum_constraint(value, constraint)
}

pub fn match_rational_constraint(value: &nmos::rational::Rational, constraint: &Value) -> bool {
    match_enum_constraint(value, constraint)
        && match_minimum_maximum_constraint(value, constraint)
}

pub fn match_constraint(value: &Value, constraint: &Value) -> bool {
    match value {
        Value::String(s) => match_string_constraint(s, constraint),
        Value::Number(n) if n.is_i64() => match_integer_constraint(n.as_i64().unwrap(), constraint),
        Value::Number(n) if n.is_f64() => match_number_constraint(n.as_f64().unwrap(), constraint),
        Value::Bool(b) => match_boolean_constraint(*b, constraint),
        _ => {
            if nmos::is_rational(value) {
                match_rational_constraint(&nmos::parse_rational(value), constraint)
            } else {
                panic!("not a valid constraint target type")
            }
        }
    }
}