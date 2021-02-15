use crate::utils::read_until_bytes;
use std::io::{prelude::*, BufReader};
use std::process::Stdio;
use std::sync::mpsc;

const PRELUDE_MARK: &[u8] = b"Prelude> ";

pub fn ghci(rx_in: mpsc::Receiver<String>, tx_out: mpsc::Sender<String>) {
    let mut process = std::process::Command::new("ghci")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = BufReader::new(process.stdout.as_mut().unwrap());

    let mut out = vec![];
    read_until_bytes(&mut stdout, PRELUDE_MARK, &mut out).unwrap();

    let (tx_err, rx_err) = mpsc::channel();
    let mut stderr = process.stderr.take();
    std::thread::spawn(move || {
        let mut err = [0; 500];
        loop {
            let n = stderr.as_mut().unwrap().read(&mut err).unwrap();
            tx_err
                .send(String::from_utf8(err[..n].to_vec()).unwrap())
                .unwrap();
        }
    });

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

        read_until_bytes(&mut stdout, PRELUDE_MARK, &mut out).unwrap();
        let out = &out[..out.len() - PRELUDE_MARK.len()];
        if out.is_empty() {
            //check stderr for errors
            tx_out.send(rx_err.try_iter().collect()).unwrap();
        } else {
            tx_out
                .send(String::from_utf8(out.to_vec()).unwrap())
                .unwrap();
        }
    }
}
