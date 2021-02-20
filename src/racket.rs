use once_cell::sync::Lazy;
use rustyline::Color;
use std::io::{prelude::*, BufReader};
use std::process::Stdio;
use std::sync::mpsc;

const PRELUDE_MARK1: &[u8] = b"\n> ";
const PRELUDE_MARK2: &[u8] = b"> ";

pub fn racket(rx_in: mpsc::Receiver<String>, tx_out: mpsc::Sender<String>) {
    let mut process = std::process::Command::new("racket")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let pid = process.id();
    let mut stdout = BufReader::new(process.stdout.as_mut().unwrap());

    let mut out = vec![];
    let mut buf = [0; 512];
    //read welcome message
    let mut read = |out: &mut Vec<u8>, mut buf: &mut [u8]| loop {
        let n = stdout.read(&mut buf).unwrap();
        out.extend(buf.iter().take(n));
        if out.ends_with(PRELUDE_MARK1) || out.ends_with(PRELUDE_MARK2) {
            break;
        }
    };
    read(&mut out, &mut buf);

    let (tx_err, rx_err) = mpsc::channel();
    let mut stderr = process.stderr.take();
    std::thread::spawn(move || {
        let mut err = [0; 500];
        loop {
            let n = stderr.as_mut().unwrap().read(&mut err).unwrap();
            let _ = tx_err.send(String::from_utf8(err[..n].to_vec()).unwrap());
        }
    });

    ctrlc::set_handler(move || {
        use nix::{
            sys::signal::{kill, Signal},
            unistd::Pid,
        };
        let _ = kill(Pid::from_raw(pid as i32), Some(Signal::SIGINT));
    })
    .expect("Error setting Ctrl-C handler");

    loop {
        out.clear();
        let inp = match rx_in.recv() {
            Ok(inp) => inp,
            // program has ended
            _ => break,
        };

        process
            .stdin
            .as_mut()
            .unwrap()
            .write_all(inp.as_bytes())
            .unwrap();

        read(&mut out, &mut buf);

        // remove "> "
        let out = String::from_utf8(out[..out.len() - 2].to_vec()).unwrap();
        let err: String = rx_err.try_iter().collect();
        tx_out.send(out + &err).unwrap();
    }
}

fn rand_color() -> (u8, u8, u8) {
    rand::random()
}

static BRACKET_COLORS: Lazy<Vec<(u8, u8, u8)>> = Lazy::new(|| {
    let mut colors = vec![];
    for _ in 0..100 {
        colors.push(rand_color());
    }
    colors
});

struct BracketColorGen {
    buffer: Vec<(u8, u8, u8)>,
    color_idx: usize,
}

impl BracketColorGen {
    fn new() -> Self {
        Self {
            buffer: Vec::new(),
            color_idx: 0,
        }
    }
    fn next(&mut self, b: char) -> String {
        let color = BRACKET_COLORS[self.color_idx];
        self.color_idx += 1;
        self.buffer.push(color);
        b.to_string().as_str().rgb(color.0, color.1, color.2)
    }
    fn rev(&mut self, b: char) -> String {
        if let Some(color) = self.buffer.pop() {
            b.to_string().as_str().rgb(color.0, color.1, color.2)
        } else {
            b.to_string()
        }
    }
}

pub fn highlight(line: &str, _pos: usize) -> String {
    let mut bracket_color = BracketColorGen::new();
    let mut colored = String::new();

    const DEFINE: &str = "define";

    let mut chars = line.chars().enumerate();

    let mut parse = || -> Option<()> {
        loop {
            let (i, c) = chars.next()?;
            match c {
                '(' => colored.push_str(&bracket_color.next('(')),
                '[' => colored.push_str(&bracket_color.next('[')),
                ')' => colored.push_str(&bracket_color.rev(')')),
                ']' => colored.push_str(&bracket_color.rev(']')),
                'd' if line.get(i..i + DEFINE.len()) == Some(DEFINE) => {
                    for _ in DEFINE.chars().skip(1) {
                        chars.next()?;
                    }
                    colored.push_str(&DEFINE.light_blue())
                }
                '+' => colored.push_str(&"+".yellow()),
                '-' => colored.push_str(&"-".yellow()),
                '/' => colored.push_str(&"/".yellow()),
                '*' => colored.push_str(&"*".yellow()),
                c => colored.push(c),
            }
        }
    };

    parse();

    colored
}
