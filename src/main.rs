use std::collections::HashSet;
use std::{borrow::Cow, sync::mpsc::channel};

use rustyline::{
    completion::Completer,
    error::ReadlineError,
    highlight::Highlighter,
    hint::Hinter,
    validate::{MatchingBracketValidator, ValidationContext, ValidationResult, Validator},
    Cmd, Context,
};
use rustyline::{Editor, Helper};

mod ghci;
use ghci::ghci;
mod smlnj;
use smlnj::smlnj;

mod utils;
use utils::StringTools;

#[derive(Default)]
struct IHsk {
    validator: MatchingBracketValidator,
    hints: HashSet<String>,
}
impl IHsk {
    fn add_to_hints(&mut self, line: &str) {
        line.split_non_alphanumeric().for_each(|item| {
            self.hints.insert(item);
        });
    }
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
//order
//1) ;
//2) key: len
fn highlight_keywords(line: &str) -> String {
    line.replace(";", "\x1b[1;33m;\x1b[0m")
        //deriving
        .replace("deriv", "\x1b[1;32mderiv\x1b[0m")
        .replace("where", "\x1b[1;31mwhere\x1b[0m")
        .replace("data", "\x1b[1;34mdata\x1b[0m")
        .replace("case", "\x1b[1;36mcase\x1b[0m")
        .replace("else", "\x1b[1;33melse\x1b[0m")
        .replace("then", "\x1b[1;33mthen\x1b[0m")
        .replace("let", "\x1b[1;32mlet\x1b[0m")
        .replace("in", "\x1b[1;32min\x1b[0m")
        .replace("of", "\x1b[1;36mof\x1b[0m")
        .replace("if", "\x1b[1;33mif\x1b[0m")
        .replace("=", "\x1b[1;33m=\x1b[0m")
        .replace("+", "\x1b[1;33m+\x1b[0m")
        .replace("-", "\x1b[1;33m-\x1b[0m")
        .replace("*", "\x1b[1;33m*\x1b[0m")
        .replace("/", "\x1b[1;33m/\x1b[0m")
        .replace("(", "\x1b[1;33m(\x1b[0m")
        .replace(")", "\x1b[1;33m)\x1b[0m")
        .replace(">", "\x1b[1;33m>\x1b[0m")
        .replace("<", "\x1b[1;33m<\x1b[0m")
        .replace("^", "\x1b[1;33m^\x1b[0m")
        .replace("|", "\x1b[1;33m|\x1b[0m")
        .replace(".", "\x1b[1;31m.\x1b[0m")
        .replace("$", "\x1b[1;31m$\x1b[0m")
}
impl Hinter for IHsk {
    type Hint = String;
}
impl Completer for IHsk {
    type Candidate = String;
    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let last_word_start_pos = line[..pos].rfind(' ').map(|i| i + 1).unwrap_or(0);
        let word_to_complete = &line[last_word_start_pos..pos];
        if word_to_complete.is_empty() {
            return Ok((0, vec![]));
        }

        let mut candidates = vec![];
        for hint in self.hints.iter() {
            if hint.starts_with(word_to_complete) {
                let hint = hint.replacen(word_to_complete, "", 1);
                candidates.push(hint);
            }
        }
        Ok((pos, candidates))
    }
}

fn main() {
    let mut rl = Editor::new();
    rl.set_helper(Some(IHsk::default()));
    rl.bind_sequence(
        rustyline::KeyEvent(rustyline::KeyCode::Char('s'), rustyline::Modifiers::CTRL),
        Cmd::Newline,
    );

    let (tx_in, rx_in) = channel();
    let (tx_out, rx_out) = channel();
    std::thread::spawn(move || smlnj(rx_in, tx_out));

    let _ = load_history(&mut rl);

    loop {
        let readline = rl.readline("\x1b[1;33mIn: \x1b[0m");

        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                rl.helper_mut().unwrap().add_to_hints(&line);
                tx_in.send(line.replace("\n", "") + "\n").unwrap();
                let out = rx_out.recv().unwrap();
                if !out.is_empty() {
                    println!("\x1b[1;31mOut:\x1b[0m {}", out);
                }
            }
            Err(ReadlineError::Interrupted) => {}
            Err(ReadlineError::Eof) => break,
            Err(_err) => break,
        }
    }
    let _ = save_history(&mut rl);
}

type CatchAll<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn load_history(rl: &mut Editor<IHsk>) -> CatchAll<()> {
    let ihsk_path = dirs_next::cache_dir().ok_or("")?.join("ihsk");
    let _ = std::fs::create_dir(&ihsk_path);

    let _ = rl.load_history(&ihsk_path.join("history.txt"));
    Ok(())
}

fn save_history(rl: &mut Editor<IHsk>) -> CatchAll<()> {
    let ihsk_path = dirs_next::cache_dir().ok_or("")?.join("ihsk");
    let _ = rl.save_history(&ihsk_path.join("history.txt"));
    Ok(())
}
