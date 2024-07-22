use std::fmt::Write;
use std::fs::File;
use std::io::{self, Read};

pub mod ansi;
use ansi::*;

pub mod ascii;
use ascii::*;

fn colorize_char(c: char) -> String {
    if c.is_ascii_alphanumeric() || "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~".contains(c) {
        format!("{}{}{}", Ansi::GREEN.code(), c, Ansi::RESET.code())
    } else {
        format!(
            "{}{}{}",
            Ansi::RED.code(),
            stringify_npc(c),
            Ansi::RESET.code()
        )
    }
}

fn colorize_and_pad(str: &String, color: &Ansi, padding: usize) -> String {
    let colored = format!("{}{}{}", color.code(), str, Ansi::RESET.code());
    let actual_length = str.len() + color.code().len() + Ansi::RESET.code().len();
    let padding = padding + actual_length;
    format!("{:<padding$}", colored)
}

struct TableOptions {
    hex: bool,
    dec: bool,
    oct: bool,
    bin: bool,
    chr: bool,
    horizontal: bool,
    color: bool,
    columns: usize,
    column_width: usize,
    separator: String,
}

impl TableOptions {
    fn new() -> Self {
        TableOptions {
            hex: true,
            dec: true,
            oct: false,
            bin: false,
            chr: true,
            horizontal: false,
            color: true,
            columns: 5,
            column_width: 40,
            separator: " | ".to_string(),
        }
    }
}

fn print_table(bytes: Vec<u8>, options: TableOptions) {
    println!();

    let TableOptions {
        hex,
        dec,
        oct,
        bin,
        chr,
        horizontal,
        color,
        columns,
        column_width,
        separator,
    } = options;

    let mut entries: Vec<String> = vec![];

    for &byte in &bytes {
        let mut entry = String::new();

        if dec {
            let dec_str = format!("{:03}", byte);

            if color {
                write!(entry, "{}", colorize_and_pad(&dec_str, &Ansi::YELLOW, 2)).unwrap();
            } else {
                write!(entry, "{:<width$}", dec_str, width = 5).unwrap();
            }
        }

        if hex {
            let hex_str = format!("0x{:02X}", byte);

            if color {
                write!(entry, "{}", colorize_and_pad(&hex_str, &Ansi::MAGENTA, 2)).unwrap();
            } else {
                write!(entry, "{:<width$}", hex_str, width = 5).unwrap();
            }
        }

        if oct {
            let oct_str = format!("0o{:03o}", byte);
            if color {
                write!(entry, "{}", colorize_and_pad(&oct_str, &Ansi::BLUE, 2)).unwrap();
            } else {
                write!(entry, "{:<width$}", oct_str, width = 4).unwrap();
            }
        }

        if bin {
            let bin_str = format!("0b{:08b}", byte);
            if color {
                write!(entry, "{}", colorize_and_pad(&bin_str, &Ansi::CYAN, 2)).unwrap();
            } else {
                write!(entry, "{:<width$}", bin_str, width = 10).unwrap();
            }
        }

        if chr {
            let char_str = if color {
                colorize_char(byte as char)
            } else {
                format!("{}", stringify_npc(byte as char))
            };

            let padding = if color { 12 } else { 4 };

            write!(entry, "{:<width$}", char_str, width = padding).unwrap();
        }

        entries.push(entry);
    }

    let row_count = (entries.len() + columns - 1) / columns;

    for irow in 0..row_count {
        for icol in 0..columns {

            let eindex = if !horizontal {
                irow + icol * row_count
            } else {
                irow * columns + icol
            };

            let separator = if icol == 0 { "" } else { &separator };

            if eindex < entries.len() {
                let entry = &entries[eindex];
                let output = format!("{} {:<width$}", separator, entry, width = column_width);
                print!("{}", output);
            } else {
                print!("{:<width$}", "", width = column_width);
            }
        }

        println!();
    }

    println!();
}

fn parse_args() -> Vec<(String, Vec<String>)> {
    let args: Vec<String> = std::env::args().collect();
    let args_with_index: Vec<(usize, String)> = args.into_iter().enumerate().collect();

    let (key_args, val_args): (Vec<(usize, String)>, Vec<(usize, String)>) =
        args_with_index.into_iter().partition(|(_, arg)| {
            (arg.starts_with("--")
                && arg
                    .strip_prefix("--")
                    .is_some_and(|s| !s.contains("--") && s.len() == arg.len() - 2))
                || ((arg.len() == 2 || arg.len() == 3) && arg.starts_with('-'))
        });

    let mut argument_pairs: Vec<(String, Vec<String>)> = Vec::with_capacity(key_args.len());
    let mut key_args_iter = key_args.into_iter().peekable();

    while let Some((karg_idx, karg)) = key_args_iter.next() {
        let next_karg_idx = key_args_iter.peek().map(|(i, _)| i);

        let karg_values = val_args
            .iter()
            .filter_map(|(varg_idx, varg)| {
                let before_next_karg = if let Some(next_karg_idx) = next_karg_idx {
                    varg_idx < next_karg_idx
                } else {
                    // No next karg index, remaining vargs belong to this karg.
                    true
                };

                // Must be after the current karg's index.
                let after_this_karg = varg_idx > &karg_idx;

                (after_this_karg && before_next_karg).then_some(varg.into())
            })
            .collect();

        argument_pairs.push((karg, karg_values));
        // argument_map.insert(karg.clone()into(), karg_values);
    }

    argument_pairs
}

const HELP_TEXT: &str = r#"
Usage: asciitty [FLAGS] [OPTIONS]

Flags <default>:
    --help       <false>        Show this help message.
    --hex        <true>         Show hexadecimal representation
    --dec        <true>         Show decimal representation
    --oct        <false>        Show octal representation
    --bin        <false>        Show binary representation
    --chr        <true>         Show raw ASCII character representation
    --horizontal <false>        Lay out bytes horizontally.
    --color      <true>         Colorize output via ANSI.
    --stdin      <false>        Read bytes from stdin.

    Flags can be negated by prefixing with 'no-', e.g. --no-hex
    to disable flags set by default.

Options <default>:
    --columns   <4>    Number of columns to display.
    --width     <40>   Width of each column.
    --separator <|>    Separator between columns.

    --bytes  <ascii>   Which bytes to display.
             ---------------------------------
             ascii | extended | asciix | 1,2,3,4 ..etc
             ---------------------------------
             When providing specific values, the following formats are
             accepted: 0xN (hex), 0oN (octal), 0bN (binary), N (decimal).
             Or a range of values: 0..127, 0x00..0x7F, etc, or any combination
             of the above, e.g. 1,0x02,0b1100,1..10 etc. Duplicates are not
             ignored, thus:

             1,0x02,0b1100,1..10 =>
                1, 2, 3, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10

             Ranges are inclusive.

             ascii, extended, and asciix can be included in the separated list
             e.g. ascii,extended would be the same as asciix, likewise, 
             so would 0..127,extended.
             --------------------------------

    --string <None>    Display bytes from a string.
    --file <None>      Read bytes from a file. 

    When using --stdin, --string, or --file, the default value of --bytes
    is ignored unless explicitly provided. If multiple sources are provided,
    the bytes are appended in the order the arguments are provided.
"#;

fn main() {
    let arguments = parse_args();
    let mut options = TableOptions::new();

    let mut input_bytes: Vec<u8> = vec![];

    for (argument, values) in arguments {
        match argument.as_str() {
            "--help" | "-h" => {
                println!("{}", HELP_TEXT);
                std::process::exit(0);
            }
            "--hex" => options.hex = true,
            "--no-hex" => options.hex = false,
            "--dec" => options.dec = true,
            "--no-dec" => options.dec = false,
            "--oct" => options.oct = true,
            "--no-oct" => options.oct = false,
            "--bin" => options.bin = true,
            "--no-bin" => options.bin = false,
            "--chr" => options.chr = true,
            "--no-chr" => options.chr = false,
            "--color" => options.color = true,
            "--horizontal" => options.horizontal = true,
            "--no-horizontal" => options.horizontal = false,
            "--no-color" => {
                options.color = false;
                options.column_width = 0;
            }

            "--stdin" => {
                let mut buffer = Vec::new();
                io::stdin().read_to_end(&mut buffer).unwrap();
                input_bytes.extend(buffer);
            }

            "--columns" if values.len() >= 1 => match values[0].parse::<usize>() {
                Ok(columns) => options.columns = columns,
                Err(_) => {
                    eprintln!("Invalid value for --columns: {}", values[0]);
                    std::process::exit(1);
                }
            },

            "--width" if values.len() >= 1 => match values[0].parse::<usize>() {
                Ok(width) => options.column_width = width,
                Err(_) => {
                    eprintln!("Invalid value for --width: {}", values[0]);
                    std::process::exit(1);
                }
            },

            "--separator" if values.len() >= 1 => {
                options.separator = values[0].clone();
            }

            "--bytes" if values.len() >= 1 => {
                let sources = values[0].as_str();
                let parts = sources.split(",");

                for part in parts {
                    match part.to_lowercase().as_str() {
                        "ascii" => input_bytes.extend(0..=127),
                        "extended" => input_bytes.extend(128..=255),
                        "asciix" => input_bytes.extend(0..=255),
                        _ => {
                            input_bytes.extend(parse_bytes(part));
                        }
                    }
                }
            }

            "--string" if values.len() >= 1 => {
                let s = values[0].as_str();
                input_bytes.extend(s.bytes());
            }

            "--file" if values.len() >= 1 => {
                let file_path = values[0].as_str();

                if !std::path::Path::new(file_path).is_file() {
                    eprintln!("No file found at path: {}", file_path);
                    std::process::exit(1);
                }

                let file_bytes = File::open(file_path)
                    .map(|mut file| {
                        let mut buffer = Vec::new();
                        file.read_to_end(&mut buffer).unwrap_or_else(|e| {
                            eprintln!("Error reading file '{}' due to error '{}'", file_path, e);
                            std::process::exit(1);
                        });
                        buffer
                    })
                    .unwrap_or_else(|e| {
                        eprintln!("Error opening file '{}' due to error '{}'", file_path, e);
                        std::process::exit(1);
                    });

                input_bytes.extend(file_bytes);
            }
            _ => {}
        }
    }

    if input_bytes.is_empty() {
        input_bytes.extend(0..=127);
    }

    print_table(input_bytes, options);
}

// Function to parse the bytes string into a vector of u8
fn parse_bytes(part: &str) -> Vec<u8> {
    let mut bytes = vec![];

    if part.contains("..") {
        let range_parts: Vec<&str> = part.split("..").collect();
        let start = parse_byte(range_parts[0]);
        let end = parse_byte(range_parts[1]);

        bytes.extend(start..=end);
    } else {
        bytes.push(parse_byte(part));
    }

    bytes
}

// Function to parse individual byte representations
fn parse_byte(s: &str) -> u8 {
    if s.starts_with("0x") {
        u8::from_str_radix(&s[2..], 16).unwrap()
    } else if s.starts_with("0o") {
        u8::from_str_radix(&s[2..], 8).unwrap()
    } else if s.starts_with("0b") {
        u8::from_str_radix(&s[2..], 2).unwrap()
    } else {
        s.parse().unwrap()
    }
}
