use std::collections::HashMap;

mod tokenize;
mod parse;


#[derive(Debug, PartialEq)]
pub enum Value {
    /// literal characters `null`
    Null,

    /// literal characters `true` or `false`
    Boolean(bool),

    /// a number, either integer or floating point
    Number(f64),

    /// a string of characters wrapped in double quotes
    String(String),

    /// an array of values
    Array(Vec<Value>),

    /// an object with key-value pairs
    Object(HashMap<String, Value>),
}
