use colorize::AnsiColor;
use itertools::{repeat_n, Itertools};
use serde_json::Value;
use std::io;
use std::io::BufRead;
use std::iter::{once, repeat};

fn dive(indent: usize, value: Value) -> String {
    match value {
        Value::Null => "(null)".to_string(),
        Value::Bool(_) | Value::Number(_) => value.to_string(),
        Value::String(s) => parse_string(indent, s),
        Value::Array(a) => {
            let bullet = (indent == 0).then_some("").unwrap_or("- ");
            a.into_iter()
                .zip(left_padding_generator(indent))
                .map(|(line, padding)| {
                    format!("{}{}{}", padding, bullet, dive(indent + bullet.len(), line))
                })
                .join("\n")
        }
        Value::Object(o) => {
            let max_indent = o.keys().map(String::len).max().unwrap_or_default();
            o.into_iter()
                .zip(left_padding_generator(indent))
                .map(|((k, v), padding)| {
                    let afterkey = String::from_iter(repeat_n(' ', max_indent - k.len()));
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

fn parse_string(indent: usize, s: String) -> String {
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
        .zip(left_padding_generator(indent))
        .map(|(line, padding)| format!("{}{}", padding, line))
        .join("\n")
}

fn left_padding_generator(indent: usize) -> impl Iterator<Item = String> {
    once(0)
        .chain(repeat(indent))
        .map(|n| String::from_iter(repeat_n(' ', n)))
}

fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines().flatten() {
        println!("{}", parse_string(0, line))
    }
}
