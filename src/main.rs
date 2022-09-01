use itertools::Itertools;
use serde_json::Value;
use std::io;
use std::io::BufRead;
use std::iter::repeat;

fn dive(indent: usize, value: Value) -> String {
    let left_padding = " ".repeat(indent);
    let mut left_padding_generator = [""].into_iter().chain(repeat(&*left_padding));

    match value {
        Value::Null => "(null)".to_string(),
        Value::Bool(_) | Value::Number(_) => value.to_string(),
        Value::String(s) => s
            .split('\n')
            .map(move |line| format!("{}{}", left_padding_generator.next().unwrap(), line))
            .join("\n"),
        Value::Array(a) => a
            .into_iter()
            .map(|line| {
                format!(
                    "{}- {}",
                    left_padding_generator.next().unwrap(),
                    dive(indent + 2, line)
                )
            })
            .join("\n"),
        Value::Object(o) => {
            let max_indent = o.keys().map(String::len).max().unwrap_or_default();
            o.into_iter()
                .map(|(k, v)| {
                    let afterkey = " ".repeat(max_indent - k.len());
                    format!(
                        "{}{}{}: {}",
                        left_padding_generator.next().unwrap(),
                        k,
                        afterkey,
                        dive(indent + max_indent + 2, v)
                    )
                })
                .join("\n")
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
