use crate::Args;

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    Scanning,
    Parsing,
    Bytecode,
    Interpreting,
}

impl Mode {
    pub fn from(args: &Args) -> Result<Self, i32> {
        match (args.scanning, args.parsing, args.bytecode) {
            (true, false, false) => Ok(Self::Scanning),
            (false, true, false) => Ok(Self::Parsing),
            (false, false, true) => Ok(Self::Bytecode),
            (false, false, false) => Ok(Self::Interpreting),
            _ => Err(1), // should be unreachable, clap rejects the combination
        }
    }
}
