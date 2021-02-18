pub trait VecTools {
    fn contains_slice(&self, slice: &[u8]) -> bool;
}
impl VecTools for Vec<u8> {
    fn contains_slice(&self, slice: &[u8]) -> bool {
        let mut idx = 0;
        loop {
            if idx == self.len() {
                return false;
            }

            if self[idx] == slice[0] {
                //check the rest immediately
                if idx + slice.len() >= self.len() {
                    return false;
                }
                if &self[idx..idx + slice.len()] == slice {
                    return true;
                }
            }
            idx += 1;
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
