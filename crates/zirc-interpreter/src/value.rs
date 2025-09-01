//! Value types for the Zirc interpreter.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A 64-bit signed integer value
    Int(i64),
    /// A UTF-8 encoded string value
    Str(String),
    /// A boolean value (true or false)
    Bool(bool),
    /// A dynamic list containing other values
    List(Vec<Value>),
    /// The unit value representing "no value"
    Unit,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            Value::List(items) => {
                write!(f, "[")?;
                for (i, it) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", it)?;
                }
                write!(f, "]")
            }
            Value::Unit => write!(f, "<unit>"),
        }
    }
}

