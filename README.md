# JSON Log pretty printer

This is rendering json logs in a way that is more readable when you are developing locally.

Say you have log lines like:
```json
{"request": "POST /api/v1/something HTTP/1.1", "response_time_ms": "2", "error.stack": "some.function\n\tsomepath/somefile:15\nsome.function\n\tsomepath/somefile:15", "status": "400", "user_agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:103.0) Gecko/20100101 Firefox/103.0"}
```

If you pipe it through `jsonlog-pretty-printer`, you will get:

```
error.stack     : some.function
                  	somepath/somefile:15
                  some.function
                  	somepath/somefile:15
request         : POST /api/v1/something HTTP/1.1
response_time_ms: 2
status          : 400
user_agent      : Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:103.0) Gecko/20100101 Firefox/103.0
```

Attempting to pretty-print via `jq` or `json_pp` won't render newlines, which is inconvenient for stack traces ; this utility takes care of that. It also indents properly which is not possible with some `sed` hack.

The indentation will be consistent when rendering a single json line, but it won't be consistent between different lines.   

Lines that do not parse as json are output as-is.

## Usage

```
Prettifies JSON logs. The log is read from the standard input

USAGE:
    jsonlog-pretty-printer [OPTIONS]

OPTIONS:
    -h, --help
            Print help information

    -t, --parse-go-test-output <PARSE_GO_TEST_OUTPUT>
            Parses the json output from gotest, aggregating the output from each test. Defaults to
            true [possible values: true, false]

    -u, --use-bold <USE_BOLD>
            Displays first-level labels in bold. Defaults to true if output is a terminal [possible
            values: true, false]

    -V, --version
            Print version information

    -w, --wrap-long-lines <WRAP_LONG_LINES>
            Wrap lines exceeding the console width. Defaults to true if output is a terminal
            [possible values: true, false]
```

## Installation

```
cargo build --release
```

then use `./target/release/jsonlog-pretty-printer` directly or copy it somewhere.  

## Usage

`jsonlog-pretty-printer` just takes the log from the standard input and outputs the rendered version to the standard output.

If you can't directly pipe your logs through `jsonlog-pretty-printer`, which may be the case if you run your application in an IDE, you can redirect the application output to a file and then pipe that file through `jsonlog-pretty-printer`.

In that case, to avoid filling your disk with application logs, you may want to create a named pipe ( `mkfifo output.log` ) and make your application output its logs to it ( i.e. 'Save console output to file' in the 'logs' tab in Jetbrains IDEs ), then `jsonlog-pretty-printer <> output.log` to render the prettified version.

Note: `<>` is used to keep the named pipe open in read-write mode ( https://stackoverflow.com/questions/63496203/keep-reading-from-a-named-pipe-after-the-first-write-has-finished )