// MIT License
//
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#[derive(Debug)]
pub enum ErrorType {
    NotFound,
    Corruption,
    NotSupported,
    InvalidArgument,
    IOError,
}

impl ErrorType {
    pub fn as_str(&self) -> &'static str {
        match *self {
            ErrorType::NotFound => "NotFoundError",
            ErrorType::Corruption => "CorruptionError",
            ErrorType::NotSupported => "NotSupportedError",
            ErrorType::InvalidArgument => "InvalidArgumentError",
            ErrorType::IOError => "IOError",
        }
    }
}

#[derive(Debug)]
pub struct Error {
    ty: ErrorType,
    msg: &'static str,
}

impl Error {
    pub fn new(ty: ErrorType, msg: &'static str) -> Error { Error { ty, msg } }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        if self.msg.is_empty() {
            write!(f, "LevelDB {}", self.ty.as_str())
        } else {
            write!(f, "LevelDB {}: {}", self.ty.as_str(), self.msg)
        }
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str { self.msg }
}

pub type Result<T> = ::std::result::Result<T, Error>;

macro_rules! LEVELDB_ERR {
    ($tp:tt) => {
        Err(Error::new(ErrorType::$tp, ""))
    };
    ($tp:tt, $msg:expr) => {
        Err(Error::new(ErrorType::$tp, $msg))
    };
}