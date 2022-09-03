use colorize::AnsiColor;
use itertools::{repeat_n, Itertools};
use serde_json::Value;
use std::io;
use std::io::BufRead;
use std::iter::{once, repeat};

fn dive(indent: usize, value: Value) -> String {
    match value {
        Value::Bool(_) | Value::Number(_) | Value::Null => value.to_string(),
        Value::String(s) => parse_string(indent, s),
        Value::Array(a) => {
            let bullet = (indent == 0).then_some("").unwrap_or("- ");
            a.into_iter()
                .map(|value| dive(indent + bullet.len(), value))
                .filter(|s| !s.is_empty())
                .zip(left_padding_generator(indent))
                .map(|(line, padding)| format!("{}{}{}", padding, bullet, line))
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
    // looking at you dd-trace-go 👀. Stop trying if there is a newline.
    for (n, c) in s.chars().enumerate() {
        match c {
            '\n' => break,
            '{' | '[' => {
                if let Ok(value) = serde_json::from_str::<Value>(&s[n..]) {
                    return dive(
                        indent,
                        Value::Array(vec![Value::String(s[..n].to_string()), value]),
                    );
                }
            }
            _ => (),
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
