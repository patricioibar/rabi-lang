use crate::Args;

pub enum Mode {
    Scanning,
    Parsing,
    Interpreting,
}

impl Mode {
    pub fn from(args: &Args) -> Result<Self, i32> {
        match (args.parsing, args.scanning) {
            (true, false) => Ok(Self::Parsing),
            (false, true) => Ok(Self::Scanning),
            (false, false) => Ok(Self::Interpreting),
            (true, true) => Err(1), // should be unreachable
        }
    }
}
