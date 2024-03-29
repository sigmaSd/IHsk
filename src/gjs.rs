use rustyline::Color;
use std::io::{prelude::*, BufReader};
use std::process::Stdio;
use std::sync::mpsc;

const PRELUDE_MARK: &[u8] = b"gjs> ";

pub fn gjs(rx_in: mpsc::Receiver<String>, tx_out: mpsc::Sender<String>) {
    let mut process = std::process::Command::new("gjs")
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
        if out.ends_with(PRELUDE_MARK) {
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

        // remove gjs>
        let out = String::from_utf8(out[..out.len() - 5].to_vec()).unwrap();
        // remove the input from the output (for some reason its included)
        let out = out.strip_prefix(&inp).unwrap();
        let err: String = rx_err.try_iter().collect();
        tx_out.send(out.to_owned() + &err).unwrap();
    }
}

pub fn highlight(line: &str, _pos: usize) -> String {
    // these chars can't be replaced \x1b[;3m0
    line.replace("exception", &"exception".red())
        .replace("datatype", &"datatype".green())
        //sig + signature
        .replace("sig", &"sig".green())
        .replace("nature", &"nature".green())
        //struct + structure
        .replace("struct", &"struct".green())
        .replace("const", &"const".rgb(194, 14, 234))
        .replace("type", &"type".green())
        .replace("raise", &"raise".red())
        .replace("case", &"case".light_blue())
        .replace("then", &"then".light_blue())
        .replace("else", &"else".light_blue())
        .replace("let", &"let".rgb(194, 14, 234))
        .replace("val", &"val".rgb(194, 14, 234))
        .replace("end", &"end".green())
        .replace("fun", &"fun".green())
        .replace("of", &"of".light_blue())
        .replace("if", &"if".light_blue())
        .replace("|", &"|".light_blue())
        .replace("(", &"(".yellow())
        .replace("*", &"*".yellow())
        .replace(")", &")".yellow())
        .replace(":", &":".yellow())
        .replace("-", &"-".yellow())
        .replace("~", &"~".yellow())
        .replace("+", &"+".yellow())
        .replace(">", &">".yellow())
        .replace("<", &"<".yellow())
        .replace("/", &"/".yellow())
        .replace("=", &"=".yellow())
        .replace(",", &",".yellow())
        .replace(".", &".".red())
}
