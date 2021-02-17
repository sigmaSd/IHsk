use crate::utils::read_until_bytes;
use std::io::{prelude::*, BufReader};
use std::process::Stdio;
use std::sync::mpsc;

const PRELUDE_MARK1: &[u8] = b"Prelude";
const PRELUDE_MARK2: &[u8] = b"> ";

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
    //read welcome message
    read_until_bytes(&mut stdout, PRELUDE_MARK1, &mut out).unwrap();
    read_until_bytes(&mut stdout, PRELUDE_MARK2, &mut out).unwrap();

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

        read_until_bytes(&mut stdout, PRELUDE_MARK1, &mut out).unwrap();
        read_until_bytes(&mut stdout, PRELUDE_MARK2, &mut out).unwrap();

        let out = String::from_utf8(out.to_vec()).unwrap();
        let end = out.rfind("Prelude").unwrap();
        let out = out[..end].to_owned();
        let err: String = rx_err.try_iter().collect();
        tx_out.send(out + &err).unwrap();
    }
}
