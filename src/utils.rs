pub fn read_until_bytes<R: std::io::BufRead + ?Sized>(
    r: &mut R,
    delim: &[u8],
    buffer: &mut Vec<u8>,
) -> std::io::Result<usize> {
    let mut read = 0;
    let mut count = 0;
    loop {
        let (done, used) = {
            let available = match r.fill_buf() {
                Ok(n) => n,
                Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            };
            match available.iter().position(|b| *b == delim[count]) {
                Some(i) => {
                    buffer.extend_from_slice(&available[..=i]);

                    count += 1;
                    if count == delim.len() {
                        (true, i + 1)
                    } else {
                        (false, i + 1)
                    }
                }
                None => {
                    count = 0;
                    buffer.extend_from_slice(available);
                    (false, available.len())
                }
            }
        };
        r.consume(used);
        read += used;
        if done || used == 0 {
            return Ok(read);
        }
    }
}
