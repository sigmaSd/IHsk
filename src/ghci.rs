use std::io::{prelude::*, BufReader};
use std::process::Stdio;
use std::sync::mpsc;

const PRELUDE_MARK1: &[u8] = b"Prelude> ";

pub fn ghci(rx_in: mpsc::Receiver<String>, tx_out: mpsc::Sender<String>) {
    let mut process = std::process::Command::new("ghci")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let pid = process.id();
    let mut stdout = BufReader::new(process.stdout.as_mut().unwrap());

    let mut out = vec![];
    let mut buf = [0; 512];

    let mut read = |out: &mut Vec<u8>, mut buf: &mut [u8]| loop {
        let n = stdout.read(&mut buf).unwrap();
        out.extend(buf.iter().take(n));
        if out.ends_with(PRELUDE_MARK1) {
            break;
        }
    };

    //read welcome message
    read(&mut out, &mut buf);

    // fix the prompt to our mark
    // so the prompt doesn't change when importing module
    process
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b":set prompt \"Prelude> \"\n")
        .unwrap();

    // read the new prompt line
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

        let out = String::from_utf8(out.to_vec()).unwrap();
        let end = out.rfind("Prelude> ").unwrap();

        //Note!!! sometimes the output starts with space????
        let out = out[..end].to_owned();

        let err: String = rx_err.try_iter().collect();
        tx_out.send(out + &err).unwrap();
    }
}

pub fn highlight(line: &str, _pos: usize) -> String {
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
