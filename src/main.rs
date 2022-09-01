use std::io;
use std::io::BufRead;
use itertools::Itertools;
use serde_json::Value;


fn dive(indent: usize, value: Value) -> String {
    match value {
        Value::Null => "(null)".to_string(),
        Value::Bool(_) | Value::Number(_) => value.to_string(),
        Value::String(s) => {
            s.split("\n").enumerate().map(|(n, line)| {
                let spaces = (n > 0).then(|| " ".repeat(indent)).unwrap_or_else(|| String::new());
                format!("{}{}", spaces, line)
            }).join("\n")
        }
        Value::Array(a) => {
            a.into_iter().enumerate().map(|(n, line)| {
                let spaces = (n > 0).then(|| " ".repeat(indent)).unwrap_or_else(|| String::new());
                format!("{}- {}", spaces, dive(indent + 2, line))
            }).join("\n")
        }
        Value::Object(o) => {
            let max_indent = o.keys().map(String::len).max().unwrap_or_default();
            o.into_iter().enumerate().map(|(n, (k, v))| {
                let beforekey = (n > 0).then(|| " ".repeat(indent)).unwrap_or_else(|| String::new());
                let afterkey = " ".repeat(max_indent - k.len());
                format!("{}{}{}: {}", beforekey, k, afterkey, dive(indent + max_indent + 2, v))
            }).join("\n")
        }
    }
}

fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines().flatten() {
        let json_line = serde_json::from_str::<serde_json::Value>(&*line);

        if let Ok(value) = json_line {
            println!("{}", dive(0, value))
        } else {
            println!("{}", line)
        }
    }
}
