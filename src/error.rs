use std::{fmt, io};

#[derive(Debug)]
pub enum Error {
    Shader { cause: String },
    Io(io::Error),
    DeviceLost,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Shader { ref cause } => writeln!(fmt, "Shader: {}", cause),
            Error::Io(ref err) => writeln!(fmt, "I/O: {}", err),
            Error::DeviceLost => writeln!(fmt, "Device Lost"),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}
