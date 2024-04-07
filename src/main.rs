use clap::Parser;
use crossterm::{
    execute,
    style::{Stylize},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},

};
use atty::Stream;
use prettytable::{format, Cell, Row, Table};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader, BufWriter, Write, stdin},
    process, thread,
};

mod args;
mod data;
mod evaluate;

use args::{CLI, Color, Decorations};
use data::VALUE_TABLE;
use evaluate::evaluate;

/// Main Function
fn main() {
    let args = CLI::parse();
    let mut output_buffer = BufWriter::new(io::stdout()); // Buffer to store output

    match args.color {
        Color::Auto => {}
        Color::Always => {crossterm::style::force_color_output(true)}
        Color::Never => {crossterm::style::force_color_output(false)}
    }

    let mut decorations = true;

    match args.decorations {
        Decorations::Auto => {}
        Decorations::Always => {decorations = true;}
        Decorations::Never => {decorations = false;}
    }

    let decorations = decorations;

    if args.table {
        if args.json {
            // Convert phf::Map into a sequence of key-value pairs to preserve order
            let json_object: Vec<(char, u8)> =
                VALUE_TABLE.entries().map(|(k, v)| (*k, *v)).collect();

            // Serialize the sequence into JSON
            let json_value: Value = json_object.into_iter().collect();
            let json_string = serde_json::to_string_pretty(&json_value).expect(
                json!("error: Failed to serialize output to json")
                    .as_str()
                    .expect("error: Failed to serialize output to json"),
            );

            writeln!(output_buffer, "{}", json_string).unwrap();
        } else if args.less || args.raw {
            for (character, value) in &VALUE_TABLE {
                writeln!(output_buffer, "{character}: {value}").unwrap();
            }
            if !args.raw || !args.quiet {
                if decorations {
                    writeln!(
                        output_buffer,
                        "{}",
                        "Note: Both lowercase and uppercase letters hold equivalent value."
                            .dark_yellow()
                            .italic()
                    )
                    .unwrap();
                } else {
                    writeln!(
                        output_buffer,
                        "{}",
                        "Note: Both lowercase and uppercase letters hold equivalent value."
                            .dark_yellow()
                    )
                        .unwrap();
                }
            }
        } else {
            let mut table = Table::new();
            table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
            if decorations {
                table.set_titles(Row::new(vec![
                    Cell::new(format!("{}", "CHARACTER".green().bold()).as_str()),
                    Cell::new(format!("{}", "VALUE".green().bold()).as_str()),
                ]));
            } else {
                table.set_titles(Row::new(vec![
                    Cell::new(format!("{}", "CHARACTER".green()).as_str()),
                    Cell::new(format!("{}", "VALUE".green()).as_str()),
                ]));
            }
            let mut last_val = '0';
            for (character, value) in &VALUE_TABLE {
                if character.is_ascii_digit() {
                    if decorations {
                        table.add_row(Row::new(vec![
                            Cell::new(
                                format!("{}", &character.to_string().dark_cyan().bold()).as_str(),
                            ),
                            Cell::new(format!("{}", &value.to_string().blue().bold()).as_str()),
                        ]));
                    } else {
                        table.add_row(Row::new(vec![
                            Cell::new(
                                format!("{}", &character.to_string().dark_cyan()).as_str(),
                            ),
                            Cell::new(format!("{}", &value.to_string().blue()).as_str()),
                        ]));
                    }
                    last_val = *character;
                } else {
                    if last_val == '9' {
                        table.add_empty_row();
                        last_val = '0';
                    }
                    if decorations {
                        table.add_row(Row::new(vec![
                            Cell::new(
                                format!("{}", &character.to_string().dark_cyan().bold()).as_str(),
                            ),
                            Cell::new(format!("{}", &value.to_string().blue().bold()).as_str()),
                        ]));
                    } else {
                        table.add_row(Row::new(vec![
                            Cell::new(
                                format!("{}", &character.to_string().dark_cyan()).as_str(),
                            ),
                            Cell::new(format!("{}", &value.to_string().blue()).as_str()),
                        ]));
                    }
                }
            }
            table
                .print(&mut output_buffer)
                .expect("error: Failed to print table");
            if !args.quiet {
                if decorations {
                    writeln!(
                        output_buffer,
                        "{}",
                        "Note: Both lowercase and uppercase letters hold equivalent value."
                            .dark_yellow()
                            .italic()
                    ).unwrap();
                } else {
                    writeln!(
                        output_buffer,
                        "{}",
                        "Note: Both lowercase and uppercase letters hold equivalent value."
                            .dark_yellow()
                    ).unwrap();
                }
            }
        }
        output_buffer.flush().unwrap();
        process::exit(0);
    }

    let mut values: Vec<u32> = vec![];

    if !atty::is(Stream::Stdin) {
        let words;
        match read_file_to_vector() {
            Ok(val) => {
                words = val;
            }
            Err(e) => {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
        if args.json {
            let mut json_output: Vec<Value> = vec![];
            let mut total = 0;

            for word in &words {
                let mut word_json = HashMap::new();
                let mut value: u32 = 0;

                for letter in word.chars() {
                    let val = evaluate(letter);
                    value += val as u32;
                }
                total += value;
                word_json.insert(word, value);
                json_output.push(json!(word_json));
            }
            if !args.no_total {
                let mut word_json = HashMap::new();
                word_json.insert("TOTAL_VALUE", total);
                json_output.push(json!(word_json));
            }

            let json_string = serde_json::to_string_pretty(&json_output).unwrap();
            writeln!(output_buffer, "{}", json_string).unwrap();
        } else {
            for word in &words {
                let mut value: u32 = 0;
                let mut letter_values: Vec<u8> = vec![];
                for letter in word.chars() {
                    let val = evaluate(letter);
                    letter_values.push(val);
                    value += val as u32;
                }
                values.push(value);
                if !args.raw {
                    if decorations {
                        writeln!(
                            output_buffer,
                            "{}",
                            format!("Value of {word:?}: {value}").blue().bold()
                        ).unwrap();
                    } else {
                        writeln!(
                            output_buffer,
                            "{}",
                            format!("Value of {word:?}: {value}").blue()
                        ).unwrap();
                    }
                } else {
                    if decorations {
                        writeln!(
                            output_buffer,
                            "{}",
                            format!("{word:?}: {value}").blue().bold()
                        )
                        .unwrap();
                    } else {
                        writeln!(
                            output_buffer,
                            "{}",
                            format!("{word:?}: {value}").blue()
                        ).unwrap();
                    }
                }
                if !args.less && !args.raw {
                    for (idx, letter) in word.chars().enumerate() {
                        writeln!(
                            output_buffer,
                            "{}",
                            format!("{letter:?}: {}", letter_values[idx]).dark_cyan()
                        )
                        .unwrap();
                    }
                }
            }
            if !args.raw {
                if decorations {
                    writeln!(output_buffer, "{}", "-".repeat(20).blue().bold()).unwrap();
                } else {
                    writeln!(output_buffer, "{}", "-".repeat(20).blue()).unwrap();
                }
            }
            if !args.no_total {
                let mut total = 0;
                for value in &values {
                    total += value;
                }
                if decorations {
                    writeln!(
                        output_buffer,
                        "{}",
                        format!("Total Value: {total}").blue().bold()
                    )
                    .unwrap();
                } else {
                    writeln!(
                        output_buffer,
                        "{}",
                        format!("Total Value: {total}").blue()
                    ).unwrap();
                }
            }
        }
        output_buffer.flush().unwrap();

        process::exit(0);
    }

    if args.recursive {
        if !args.quiet {
            if decorations {
                writeln!(
                    output_buffer,
                    "{}",
                    "Note: Press Ctrl+C to exit.".dark_yellow().italic()
                )
                .unwrap();
            } else {
                writeln!(
                    output_buffer,
                    "{}",
                    "Note: Press Ctrl+C to exit.".dark_yellow()
                ).unwrap();
            }
        }
        let mut words = args.words.clone();
        if args.json {
            loop {
                let mut json_output: Vec<Value> = vec![];
                let mut total = 0;

                for word in &words {
                    let mut word_json = HashMap::new();
                    let mut value: u32 = 0;

                    for letter in word.chars() {
                        let val = evaluate(letter);
                        value += val as u32;
                    }
                    total += value;
                    word_json.insert(word, value);
                    json_output.push(json!(word_json));
                }
                if !args.no_total {
                    let mut word_json = HashMap::new();
                    word_json.insert("TOTAL_VALUE", total);
                    json_output.push(json!(word_json));
                }

                let json_string = serde_json::to_string_pretty(&json_output).unwrap();
                writeln!(output_buffer, "{}", json_string).unwrap();
                words.clear();

                if !args.raw && !args.less {
                    write!(output_buffer, "Enter words separated by spaces: ").unwrap();
                } else if args.less {
                    write!(output_buffer, ": ").unwrap();
                } else {
                    write!(output_buffer, ":").unwrap();
                }

                let mut input = String::new();
                output_buffer.flush().unwrap();
                stdin().read_line(&mut input).expect("error: Failed to read line");

                // Trim the trailing newline character
                input = input.trim().to_string();

                // Split the input string into multiple strings
                words = input.split_whitespace().map(|s| s.to_string()).collect();
            }
        } else {
            loop {
                for word in &words {
                    let mut value: u32 = 0;
                    let mut letter_values: Vec<u8> = vec![];
                    for letter in word.chars() {
                        let val = evaluate(letter);
                        letter_values.push(val);
                        value += val as u32;
                    }
                    values.push(value);
                    if !args.raw {
                        if decorations {
                            writeln!(
                                output_buffer,
                                "{}",
                                format!("Value of {word:?}: {value}").blue().bold()
                            )
                            .unwrap();
                        } else {
                            writeln!(
                                output_buffer,
                                "{}",
                                format!("Value of {word:?}: {value}").blue()
                            ).unwrap();
                        }
                    } else {
                        if decorations {
                            writeln!(
                                output_buffer,
                                "{}",
                                format!("{word:?}: {value}\n").blue().bold()
                            )
                            .unwrap();
                        } else {
                            writeln!(
                                output_buffer,
                                "{}",
                                format!("{word:?}: {value}\n").blue()
                            ).unwrap();
                        }
                    }
                    if !args.less && !args.raw {
                        for (idx, letter) in word.chars().enumerate() {
                            writeln!(
                                output_buffer,
                                "{}",
                                format!("{letter:?}: {}", letter_values[idx]).dark_cyan()
                            )
                            .unwrap();
                        }
                    }
                }

                if !args.no_total {
                    if !args.raw {
                        if decorations {
                            writeln!(output_buffer, "{}", "-".repeat(20).blue().bold()).unwrap();
                        } else {
                            writeln!(output_buffer, "{}", "-".repeat(20).blue()).unwrap();
                        }
                    }
                    let mut total = 0;
                    for value in &values {
                        total += value;
                    }
                    if decorations {
                        writeln!(
                            output_buffer,
                            "{}",
                            format!("Total Value: {total}").blue().bold()
                        ).unwrap();
                    } else {
                        writeln!(
                            output_buffer,
                            "{}",
                            format!("Total Value: {total}").blue()
                        ).unwrap();
                    }
                }

                words.clear();
                if !args.raw && !args.less {
                    write!(output_buffer, "Enter words separated by spaces: ").unwrap();
                } else if args.less {
                    write!(output_buffer, ": ").unwrap();
                } else {
                    write!(output_buffer, ":").unwrap();
                }

                output_buffer.flush().unwrap();
                let mut input = String::new();
                stdin().read_line(&mut input).expect("error: Failed to read line");

                // Trim the trailing newline character
                input = input.trim().to_string();

                // Split the input string into multiple strings
                words = input.split_whitespace().map(|s| s.to_string()).collect();

                // Reset values
                values.clear();
            }
        }
    } else if args.fast || args.json || cfg!(windows) {
        if args.json {
            let mut json_output: Vec<Value> = vec![];
            let mut total = 0;

            for word in &args.words {
                let mut word_json = HashMap::new();
                let mut value: u32 = 0;

                for letter in word.chars() {
                    let val = evaluate(letter);
                    value += val as u32;
                }
                total += value;
                word_json.insert(word, value);
                json_output.push(json!(word_json));
            }
            if !args.no_total {
                let mut word_json = HashMap::new();
                word_json.insert("TOTAL_VALUE", total);
                json_output.push(json!(word_json));
            }

            let json_string = serde_json::to_string_pretty(&json_output).unwrap();
            writeln!(output_buffer, "{}", json_string).unwrap();
        } else {
            for word in &args.words {
                let mut value: u32 = 0;
                let mut letter_values: Vec<u8> = vec![];
                for letter in word.chars() {
                    let val = evaluate(letter);
                    letter_values.push(val);
                    value += val as u32;
                }
                values.push(value);
                if !args.raw {
                    if decorations {
                        writeln!(
                            output_buffer,
                            "{}",
                            format!("Value of {word:?}: {value}").blue().bold()
                        )
                        .unwrap();
                    } else {
                        writeln!(
                            output_buffer,
                            "{}",
                            format!("Value of {word:?}: {value}").blue()
                        ).unwrap();
                    }
                } else {
                    if decorations {
                        writeln!(
                            output_buffer,
                            "{}",
                            format!("{word:?}: {value}").blue().bold()
                        )
                        .unwrap();
                    } else {
                        writeln!(
                            output_buffer,
                            "{}",
                            format!("{word:?}: {value}").blue()
                        ).unwrap();
                    }
                }
                if !args.less && !args.raw {
                    for (idx, letter) in word.chars().enumerate() {
                        writeln!(
                            output_buffer,
                            "{}",
                            format!("{letter:?}: {}", letter_values[idx]).dark_cyan()
                        )
                        .unwrap();
                    }
                }
            }
            if !args.no_total {
                if !args.raw {
                    if decorations {
                        writeln!(output_buffer, "{}", "-".repeat(20).blue().bold()).unwrap();
                    } else {
                        writeln!(output_buffer, "{}", "-".repeat(20).blue()).unwrap();
                    }
                }
                let mut total = 0;
                for value in values {
                    total += value;
                }
                if decorations {
                    writeln!(
                        output_buffer,
                        "{}",
                        format!("Total Value: {total}").blue().bold()
                    )
                    .unwrap();
                } else {
                    writeln!(
                        output_buffer,
                        "{}",
                        format!("Total Value: {total}").blue()
                    ).unwrap();
                }
            }
        }
        output_buffer.flush().unwrap();
    } else {
        #[cfg(not(target_os = "windows"))]
        {
            // Enter alternate screen buffer
            execute!(io::stdout(), EnterAlternateScreen).expect("error: Failed to create new screen buffer");
            // Clear the terminal
            execute!(io::stdout(), terminal::Clear(terminal::ClearType::All))
                .expect("error: Failed to clear screen");

            let mut words = args.words.clone();
            setup_ctrl_c_handler();
            let ascii_art = r#"
             _   _ _   _ __  __ _____ ____      _    ____    _    _     ____
            | \ | | | | |  \/  | ____|  _ \    / \  / ___|  / \  | |   / ___|
            |  \| | | | | |\/| |  _| | |_) |  / _ \| |     / _ \ | |  | |
            | |\  | |_| | |  | | |___|  _ <  / ___ \ |___ / ___ \| |__| |___
            |_| \_|\___/|_|  |_|_____|_| \_\/_/   \_\____/_/   \_\_____\____|
            "#;
            if decorations {
                writeln!(output_buffer, "{}\n\n", ascii_art.green().bold()).unwrap();
            } else {
                writeln!(output_buffer, "{}\n\n", ascii_art.green()).unwrap();
            }
            if !args.quiet {
                if decorations {
                    writeln!(
                        output_buffer,
                        "{}",
                        "Note: Press Ctrl+C to exit.".dark_yellow().italic()
                    )
                    .unwrap();
                } else {
                    writeln!(
                        output_buffer,
                        "{}",
                        "Note: Press Ctrl+C to exit.".dark_yellow()
                    ).unwrap();
                }
            }
            loop {
                for word in &words {
                    let mut value: u32 = 0;
                    let mut letter_values: Vec<u8> = vec![];
                    for letter in word.chars() {
                        let val = evaluate(letter);
                        letter_values.push(val);
                        value += val as u32;
                    }
                    values.push(value);
                    if !args.raw {
                        if decorations {
                            writeln!(
                                output_buffer,
                                "{}",
                                format!("Value of {word:?}: {value}").blue().bold()
                            )
                            .unwrap();
                        } else {
                            writeln!(
                                output_buffer,
                                "{}",
                                format!("Value of {word:?}: {value}").blue()
                            ).unwrap();
                        }
                    } else {
                        if decorations {
                            writeln!(
                                output_buffer,
                                "{}",
                                format!("{word:?}: {value}\n").blue().bold()
                            )
                            .unwrap();
                        } else {
                            writeln!(
                                output_buffer,
                                "{}",
                                format!("{word:?}: {value}\n").blue()
                            ).unwrap();
                        }
                    }
                    if !args.less && !args.raw {
                        for (idx, letter) in word.chars().enumerate() {
                            writeln!(
                                output_buffer,
                                "{}",
                                format!("{letter:?}: {}", letter_values[idx]).dark_cyan()
                            )
                            .unwrap();
                        }
                    }
                }

                if !args.no_total {
                    if !args.raw {
                        if decorations {
                            writeln!(output_buffer, "{}", "-".repeat(20).blue().bold()).unwrap();
                        } else {
                            writeln!(output_buffer, "{}", "-".repeat(20).blue()).unwrap();
                        }
                    }
                    let mut total = 0;
                    for value in &values {
                        total += value;
                    }
                    if decorations {
                        writeln!(
                            output_buffer,
                            "{}",
                            format!("Total Value: {total}").blue().bold()
                        )
                            .unwrap();
                    } else {
                        writeln!(
                            output_buffer,
                            "{}",
                            format!("Total Value: {total}").blue()
                        )
                            .unwrap();
                    }
                }

                words.clear();
                if !args.raw && !args.less {
                    write!(output_buffer, "Enter words separated by spaces: ").unwrap();
                } else if args.less {
                    write!(output_buffer, ": ").unwrap();
                } else {
                    write!(output_buffer, ":").unwrap();
                }

                output_buffer.flush().unwrap();
                let mut input = String::new();
                stdin().read_line(&mut input).expect("error: Failed to read line");

                // Trim the trailing newline character
                input = input.trim().to_string();

                // Split the input string into multiple strings
                words = input.split_whitespace().map(|s| s.to_string()).collect();

                // Reset values
                values.clear();
            }
        }
    }
}

/// Set up a Ctrl+C signal handler
#[cfg(not(target_os = "windows"))]
fn setup_ctrl_c_handler() {
    // Use signal-hook crate to handle Ctrl+C signal
    let mut signals = signal_hook::iterator::Signals::new(&[signal_hook::consts::SIGINT])
        .expect("error: Failed to setup Ctrl+C handler");

    // Spawn a separate thread to handle the signal
    thread::spawn(move || {
        for _ in signals.forever() {
            // Clear the terminal again before exiting
            if let Err(err) = execute!(io::stdout(), terminal::Clear(terminal::ClearType::All)) {
                eprintln!("error: Failed to clear terminal: {err}");
                process::exit(1);
            }
            // Leave alternate screen buffer
            if let Err(err) = execute!(io::stdout(), LeaveAlternateScreen) {
                eprintln!("error: Failed to leave alternate buffer: {err}");
                process::exit(1);
            }
            process::exit(0);
        }
    });
}

/// Read the Words from <stdin>
fn read_file_to_vector() -> Result<Vec<String>, io::Error> {
    let reader = BufReader::new(stdin().lock());
    let mut words = Vec::new();

    for line in reader.lines() {
        let line = line?;
        for word in line.split_whitespace() {
            words.push(word.to_string());
        }
    }

    Ok(words)
}
