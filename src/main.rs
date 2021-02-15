use std::{borrow::Cow, sync::mpsc::channel};

use rustyline::{
    completion::Completer,
    error::ReadlineError,
    highlight::Highlighter,
    hint::Hinter,
    validate::{MatchingBracketValidator, ValidationContext, ValidationResult, Validator},
    Cmd,
};
use rustyline::{Editor, Helper};

mod ghci;
use ghci::ghci;
mod utils;

#[derive(Default)]
struct IHsk {
    validator: MatchingBracketValidator,
}
impl Helper for IHsk {}
impl Validator for IHsk {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<ValidationResult, ReadlineError> {
        self.validator.validate(ctx)
    }
}
impl Highlighter for IHsk {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        highlight_keywords(line).into()
    }
    fn highlight_char(&self, _line: &str, _pos: usize) -> bool {
        true
    }
}
fn highlight_keywords(line: &str) -> String {
    line.replace(";", "\x1b[1;33m;\x1b[0m")
        .replace("let", "\x1b[1;32mlet\x1b[0m")
        .replace("where", "\x1b[1;31mwhere\x1b[0m")
        .replace("=", "\x1b[1;33m=\x1b[0m")
        .replace("+", "\x1b[1;33m+\x1b[0m")
        .replace("-", "\x1b[1;33m-\x1b[0m")
        .replace("*", "\x1b[1;33m*\x1b[0m")
        .replace("/", "\x1b[1;33m/\x1b[0m")
}
impl Hinter for IHsk {
    type Hint = String;
}
impl Completer for IHsk {
    type Candidate = String;
}

fn main() {
    let mut rl = Editor::new();
    rl.set_helper(Some(IHsk::default()));
    rl.bind_sequence(
        rustyline::KeyEvent(rustyline::KeyCode::Char('s'), rustyline::Modifiers::CTRL),
        Cmd::Newline,
    );

    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    let (tx_in, rx_in) = channel();
    let (tx_out, rx_out) = channel();
    std::thread::spawn(move || ghci(rx_in, tx_out));

    loop {
        let readline = rl.readline("\x1b[1;33mIn: \x1b[0m");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                tx_in.send(line.replace("\n", "") + "\n").unwrap();
                let out = rx_out.recv().unwrap();
                if !out.is_empty() {
                    println!("\x1b[1;31mOut: \x1b[0m {}", out);
                }
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(_err) => break,
        }
    }
    rl.save_history("history.txt").unwrap();
}
