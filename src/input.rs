use std::{
    fmt::Display,
    io::{BufRead, BufReader, Cursor, Read, Result, Stdin, Write},
};

pub trait Input {
    fn read_line(&mut self, buf: &mut String) -> Result<usize>;
}

impl Input for Stdin {
    fn read_line(&mut self, buf: &mut String) -> Result<usize> {
        Stdin::read_line(self, buf)
    }
}

impl<T: AsRef<[u8]>> Input for Cursor<T> {
    fn read_line(&mut self, buf: &mut String) -> Result<usize> {
        BufRead::read_line(self, buf)
    }
}

impl<R: Read> Input for BufReader<R> {
    fn read_line(&mut self, buf: &mut String) -> Result<usize> {
        BufRead::read_line(self, buf)
    }
}

/// Writes `prompt` to `output` without a trailing newline, then reads one line
/// from `input`. Returns the line without its terminator, or `None` at end of input.
pub fn read_line(
    output: &mut impl Write,
    input: &mut dyn Input,
    prompt: impl Display,
) -> std::result::Result<Option<String>, String> {
    write!(output, "{}", prompt).map_err(|e| e.to_string())?;
    output.flush().map_err(|e| e.to_string())?;

    let mut line = String::new();
    if input.read_line(&mut line).map_err(|e| e.to_string())? == 0 {
        return Ok(None);
    }
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
    Ok(Some(line))
}
