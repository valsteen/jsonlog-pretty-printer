use colorize::AnsiColor;
use itertools::Itertools;
use serde_json::Value;
use std::io;
use std::io::BufRead;
use std::iter::repeat;

fn dive(indent: usize, value: Value) -> String {
    let left_padding = " ".repeat(indent);
    let left_padding_generator = [""].into_iter().chain(repeat(&*left_padding));

    match value {
        Value::Null => "(null)".to_string(),
        Value::Bool(_) | Value::Number(_) => value.to_string(),
        Value::String(s) => {
            if let Ok(parsed) = serde_json::from_str::<Value>(&*s) {
                dive(indent, parsed)
            } else {
                s.split('\n')
                    .zip(left_padding_generator)
                    .map(|(line, padding)| format!("{}{}", padding, line))
                    .join("\n")
            }
        }
        Value::Array(a) => a
            .into_iter()
            .zip(left_padding_generator)
            .map(|(line, padding)| format!("{}- {}", padding, dive(indent + 2, line)))
            .join("\n"),
        Value::Object(o) => {
            let max_indent = o.keys().map(String::len).max().unwrap_or_default();
            o.into_iter()
                .zip(left_padding_generator)
                .map(|((k, v), padding)| {
                    let afterkey = " ".repeat(max_indent - k.len());
                    let key = (indent == 0).then(|| k.clone().bold()).unwrap_or(k);
                    format!(
                        "{}{}{}: {}",
                        padding,
                        key,
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
