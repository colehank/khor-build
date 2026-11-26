//! Bounded approximation of a shell prompt's editable input.

const LIMIT: usize = 1024;

#[derive(Debug, Default)]
pub(super) struct InputDraft {
    text: Vec<char>,
    cursor: usize,
    certain: bool,
}

impl InputDraft {
    pub(super) fn reset(&mut self) {
        self.text.clear();
        self.cursor = 0;
        self.certain = true;
    }

    pub(super) fn discard(&mut self) {
        self.reset();
        self.certain = false;
    }

    pub(super) fn replace(&mut self, line: &str, utf16_cursor: usize) {
        self.reset();
        let mut offset = 0;
        for ch in line.chars().take(LIMIT) {
            offset += ch.len_utf16();
            if offset <= utf16_cursor {
                self.cursor += 1;
            }
            self.text.push(ch);
        }
    }

    pub(super) fn excerpt(&self) -> Option<String> {
        (self.certain && !self.text.is_empty()).then(|| self.text.iter().collect())
    }

    /// Returns true when the prompt may have been submitted or left.
    pub(super) fn keys(&mut self, bytes: &[u8]) -> bool {
        if bytes.iter().any(|b| matches!(*b, b'\r' | b'\n' | 3))
            || (self.text.is_empty() && bytes == b"\x04")
        {
            self.discard();
            return true;
        }
        if !self.certain {
            return false;
        }
        let Ok(mut keys) = std::str::from_utf8(bytes) else {
            self.discard();
            return false;
        };
        while !keys.is_empty() {
            let ch = keys.chars().next().unwrap();
            keys = &keys[ch.len_utf8()..];
            match ch {
                '\x01' => self.cursor = 0,
                '\x05' => self.cursor = self.text.len(),
                '\x02' => self.left(),
                '\x06' => self.right(),
                '\x15' => {
                    self.text.drain(..self.cursor);
                    self.cursor = 0;
                }
                '\x0b' => self.text.truncate(self.cursor),
                '\x04' => self.delete(),
                '\x08' | '\x7f' if self.cursor > 0 => {
                    self.cursor -= 1;
                    self.delete();
                }
                '\x08' | '\x7f' => {}
                '\x1b' => {
                    let Some(rest) = self.escape(keys) else {
                        self.discard();
                        break;
                    };
                    keys = rest;
                }
                ch if !ch.is_control() && self.text.len() < LIMIT => {
                    self.text.insert(self.cursor, ch);
                    self.cursor += 1;
                }
                _ => {
                    self.discard();
                    break;
                }
            }
        }
        false
    }

    fn left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    fn right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.text.len());
    }

    fn delete(&mut self) {
        if self.cursor < self.text.len() {
            self.text.remove(self.cursor);
        }
    }

    fn escape<'a>(&mut self, keys: &'a str) -> Option<&'a str> {
        let after = keys.strip_prefix('[').or_else(|| keys.strip_prefix('O'))?;
        match after.as_bytes().first()? {
            b'D' => self.left(),
            b'C' => self.right(),
            b'H' => self.cursor = 0,
            b'F' => self.cursor = self.text.len(),
            _ => {
                if let Some(rest) = after.strip_prefix("3~") {
                    self.delete();
                    return Some(rest);
                }
                return after
                    .strip_prefix("200~")
                    .or_else(|| after.strip_prefix("201~"));
            }
        }
        Some(&after[1..])
    }
}
