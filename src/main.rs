use clap::Parser;
use colorize::AnsiColor;
use itertools::{repeat_n, Either, Itertools};
use serde_json::Value;
use std::io;
use std::io::BufRead;
use std::iter::{once, repeat};

/// Prettifies JSON logs. The log is read from the standard input.
#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    /// Wrap lines exceeding the console width. Defaults to true if output is a terminal.
    #[clap(short, long, value_parser)]
    wrap_long_lines: Option<bool>,

    /// Displays first-level labels in bold. Defaults to true if output is a terminal.
    #[clap(short, long, value_parser)]
    use_bold: Option<bool>,
}

struct Prettifier {
    width: Option<u16>,
    use_bold: bool,
}

impl Prettifier {
    fn dive(&self, indent: usize, value: Value) -> String {
        match value {
            Value::Bool(_) | Value::Number(_) | Value::Null => value.to_string(),
            Value::String(s) => self.parse_string(indent, s),
            Value::Array(a) => {
                let bullet = (indent == 0).then_some("").unwrap_or("- ");
                a.into_iter()
                    .map(|value| self.dive(indent + bullet.len(), value))
                    .filter(|s| !s.is_empty())
                    .zip(left_padding_generator(indent))
                    .map(|(line, padding)| format!("{padding}{bullet}{line}"))
                    .join("\n")
            }
            Value::Object(o) => {
                let max_indent = o.keys().map(String::len).max().unwrap_or_default();
                o.into_iter()
                    .zip(left_padding_generator(indent))
                    .map(|((k, v), padding)| {
                        let afterkey = String::from_iter(repeat_n(' ', max_indent - k.len()));
                        let key = (indent == 0).then(|| self.bold(k.clone())).unwrap_or(k);
                        format!(
                            "{padding}{key}{afterkey}: {dive}",
                            dive = self.dive(indent + max_indent + 2, v)
                        )
                    })
                    .join("\n")
            }
        }
    }

    fn parse_string(&self, indent: usize, s: String) -> String {
        // try hard to parse JSON strings shoved in regular strings
        // looking at you dd-trace-go 👀. Stop trying if there is a newline.
        for (n, c) in s.chars().enumerate() {
            match c {
                '\n' => break,
                '{' | '[' => {
                    if let Ok(value) = serde_json::from_str::<Value>(&s[n..]) {
                        return self.dive(
                            indent,
                            Value::Array(vec![Value::String(s[..n].to_string()), value]),
                        );
                    }
                }
                _ => (),
            }
        }

        let lines = s.split('\n');

        let lines = if let Some(width) = self
            .width
            .map(|w| w as i32 - indent as i32)
            .filter(|w| *w > 10)
        {
            Either::Left(lines.flat_map(move |line| {
                line.chars()
                    .flat_map(|c| match c {
                        '\t' => Either::Left("    ".chars()),
                        _ => Either::Right([c].into_iter()),
                    })
                    .chunks(width as usize)
                    .into_iter()
                    .map(String::from_iter)
                    .collect_vec()
            }))
        } else {
            Either::Right(lines.map(String::from))
        };

        lines
            .zip(left_padding_generator(indent))
            .map(|(line, padding)| format!("{padding}{line}"))
            .join("\n")
    }

    fn bold(&self, s: String) -> String {
        if self.use_bold {
            s.bold()
        } else {
            s
        }
    }
}

fn left_padding_generator(indent: usize) -> impl Iterator<Item = String> {
    once(0)
        .chain(repeat(indent))
        .map(|n| String::from_iter(repeat_n(' ', n)))
}

fn main() {
    let args: Args = Args::parse();
    let is_tty = termion::is_tty(&io::stdout());

    let mut prettifier = Prettifier {
        width: None,
        use_bold: false,
    };

    prettifier.use_bold = matches!((args.use_bold, is_tty), (None, true) | (Some(true), _));

    let get_width = match (args.wrap_long_lines, is_tty) {
        (None, true) | (Some(true), _) => || termion::terminal_size().ok().map(|(w, _)| w),
        _ => || None,
    };

    let stdin = io::stdin();
    for line in stdin.lock().lines().flatten() {
        prettifier.width = get_width();
        println!("{}", prettifier.parse_string(0, line))
    }
}
