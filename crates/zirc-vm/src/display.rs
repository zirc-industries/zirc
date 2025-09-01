//! Pretty-printer for VM values.

use zirc_bytecode::Value;

pub fn display_value(v: &Value) -> String {
    match v {
        Value::Int(n) => n.to_string(),
        Value::Str(s) => s.clone(),
        Value::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
        Value::List(items) => {
            let mut s = String::from("[");
            for (i, it) in items.iter().enumerate() {
                if i > 0 { s.push_str(", "); }
                s.push_str(&display_value(it));
            }
            s.push(']');
            s
        }
        Value::Unit => "<unit>".to_string(),
    }
}

