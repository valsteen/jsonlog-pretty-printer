use colorize::AnsiColor;
use itertools::{repeat_n, Itertools};
use serde_json::Value;
use std::io;
use std::io::BufRead;
use std::iter::{once, repeat};

fn dive(indent: usize, value: Value) -> String {
    let left_padding_generator = once(0)
        .chain(repeat(indent))
        .map(|n| repeat_n(' ', n).collect::<String>());
    match value {
        Value::Null => "(null)".to_string(),
        Value::Bool(_) | Value::Number(_) => value.to_string(),
        Value::String(s) => {
            // try hard to parse JSON strings shoved in regular strings
            // looking at you dd-trace-go 👀
            for (position, c) in s.chars().enumerate() {
                if let Some(Ok(value)) =
                    ("{[".contains(c)).then(|| serde_json::from_str::<Value>(&s[position..]))
                {
                    let dive_into = if position == 0 {
                        value
                    } else {
                        Value::Array(vec![Value::String(s[..position].to_string()), value])
                    };
                    return dive(indent, dive_into);
                }
            }

            s.split('\n')
                .zip(left_padding_generator)
                .map(|(line, padding)| format!("{}{}", padding, line))
                .join("\n")
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
        let json_line = serde_json::from_str::<Value>(&*line);

        if let Ok(value) = json_line {
            println!("{}", dive(0, value))
        } else {
            println!("{}", line)
        }
    }
}
