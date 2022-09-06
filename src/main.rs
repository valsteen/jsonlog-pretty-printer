use clap::Parser;
use colorize::AnsiColor;
use itertools::{repeat_n, Either, Itertools};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
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

    /// Parses the json output from gotest, aggregating the output from each test. Defaults to true
    #[clap(short('t'), long, value_parser)]
    parse_go_test_output: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
#[serde(rename_all = "lowercase")]
enum GoTestAction {
    Pass,
    Fail,
    Output,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Debug)]
struct GoTestKey {
    package: Option<String>,
    test: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct GoTestEntry {
    time: String,
    action: GoTestAction,
    package: Option<String>,
    test: Option<String>,
    output: Option<String>,
}

struct Prettifier {
    width: Option<u16>,
    use_bold: bool,
    go_test_entries: HashMap<GoTestKey, String>,
    parse_go_test_output: bool,
}

impl Prettifier {
    fn dive(&mut self, indent: usize, value: Value) -> Option<String> {
        let result = match value {
            Value::Bool(_) | Value::Number(_) | Value::Null => value.to_string(),
            Value::String(s) => return self.parse_string(indent, s),
            Value::Array(a) => {
                let bullet = (indent == 0).then_some("").unwrap_or("- ");
                a.into_iter()
                    .flat_map(|value| self.dive(indent + bullet.len(), value))
                    .filter(|s| !s.is_empty())
                    .zip(left_padding_generator(indent))
                    .map(|(line, padding)| format!("{padding}{bullet}{line}"))
                    .join("\n")
            }
            Value::Object(o) => {
                let max_indent = o.keys().map(String::len).max().unwrap_or_default();
                o.into_iter()
                    .zip(left_padding_generator(indent))
                    .flat_map(|((key, value), padding)| {
                        self.dive(indent + max_indent + 2, value).map(|value| {
                            let afterkey = String::from_iter(repeat_n(' ', max_indent - key.len()));
                            let key = (indent == 0).then(|| self.bold(key.clone())).unwrap_or(key);
                            format!("{padding}{key}{afterkey}: {value}")
                        })
                    })
                    .join("\n")
            }
        };
        Some(result)
    }

    fn parse_go_test_entry(&mut self, go_test_entry: GoTestEntry) -> Option<String> {
        let key = GoTestKey {
            package: go_test_entry.package.clone(),
            test: go_test_entry.test.clone(),
        };

        match &go_test_entry.action {
            GoTestAction::Fail | GoTestAction::Pass => {
                let output = self.go_test_entries.remove(&key);
                let mut map = serde_json::Map::new();
                if let Some(package) = key.package {
                    map.insert("Package".to_string(), Value::String(package));
                }
                if let Some(test) = key.test {
                    map.insert("Test".to_string(), Value::String(test));
                }
                if let Some(output) = output {
                    let test_output = output
                        .split('\n')
                        .flat_map(|s| self.parse_string(0, s.to_string()))
                        .join("\n");
                    map.insert("Output".to_string(), Value::String(test_output));
                }
                map.insert("Time".to_string(), Value::String(go_test_entry.time));
                if let Ok(action) = serde_json::to_string(&go_test_entry.action) {
                    map.insert("Action".to_string(), Value::String(action));
                }
                self.dive(0, Value::Object(map))
            }
            GoTestAction::Output => {
                let key = GoTestKey {
                    package: go_test_entry.package.clone(),
                    test: go_test_entry.test.clone(),
                };
                if let Some(output) = go_test_entry.output {
                    self.go_test_entries
                        .entry(key)
                        .or_default()
                        .push_str(&*output);
                }
                None
            }
        }
    }

    fn parse_string(&mut self, indent: usize, s: String) -> Option<String> {
        if self.parse_go_test_output && indent == 0 {
            if let Ok(go_test_entry) = serde_json::from_str::<GoTestEntry>(&*s) {
                return self.parse_go_test_entry(go_test_entry);
            }
        }

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

        Some(
            lines
                .zip(left_padding_generator(indent))
                .map(|(line, padding)| format!("{padding}{line}"))
                .join("\n"),
        )
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
        go_test_entries: HashMap::new(),
        parse_go_test_output: args.parse_go_test_output.unwrap_or(true),
    };

    prettifier.use_bold = matches!((args.use_bold, is_tty), (None, true) | (Some(true), _));

    let get_width = match (args.wrap_long_lines, is_tty) {
        (None, true) | (Some(true), _) => || termion::terminal_size().ok().map(|(w, _)| w),
        _ => || None,
    };

    let stdin = io::stdin();
    for line in stdin.lock().lines().flatten() {
        if let Some(line) = prettifier.parse_string(0, line) {
            prettifier.width = get_width();
            println!("{}", line)
        }
    }
}
