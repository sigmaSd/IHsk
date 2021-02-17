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

pub trait StringTools {
    fn split_non_alphanumeric<'a>(&'a self) -> Box<dyn Iterator<Item = String> + 'a>;
}

impl StringTools for &str {
    fn split_non_alphanumeric<'a>(&'a self) -> Box<dyn Iterator<Item = String> + 'a> {
        Box::new(SplitNonAlphanumeric::new(self.chars().peekable()))
    }
}

struct SplitNonAlphanumeric<I: Iterator<Item = char>> {
    buffer: I,
    part: String,
}

impl<I: Iterator<Item = char>> SplitNonAlphanumeric<I> {
    fn new(buffer: I) -> Self {
        Self {
            buffer,
            part: String::new(),
        }
    }
}

impl<I: Iterator<Item = char>> Iterator for SplitNonAlphanumeric<I> {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // we may finish here
            // return part if its not empty
            let c = match self.buffer.next() {
                Some(c) => c,
                None => {
                    if self.part.is_empty() {
                        return None;
                    } else {
                        return Some(self.part.drain(..).collect());
                    }
                }
            };

            if c.is_alphanumeric() || c == '_' {
                self.part.push(c);
            } else {
                break;
            }
        }
        // here we're on a non aplhanumeric character
        if !self.part.is_empty() {
            // if we read something yield it
            Some(self.part.drain(..).collect())
        } else {
            // else drain all next non aplhanumeric characters
            // and then recurse to try to find the next part
            while let Some(c) = self.buffer.next() {
                if c.is_alphanumeric() || c == '_' {
                    self.part.push(c);
                    return self.next();
                }
            }
            // finish
            None
        }
    }
}
