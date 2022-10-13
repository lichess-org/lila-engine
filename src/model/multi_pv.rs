use std::fmt;
use thiserror::Error;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct MultiPv(u32);

impl Default for MultiPv {
    fn default() -> MultiPv {
        MultiPv(1)
    }
}

#[derive(Error, Debug)]
#[error("supported range is 1 to 5")]
pub struct InvalidMultiPvError;

impl TryFrom<u32> for MultiPv {
    type Error = InvalidMultiPvError;

    fn try_from(n: u32) -> Result<MultiPv, InvalidMultiPvError> {
        if 1 <= n && n <= 5 {
            Ok(MultiPv(n))
        } else {
            Err(InvalidMultiPvError)
        }
    }
}

impl From<MultiPv> for u32 {
    fn from(MultiPv(n): MultiPv) -> u32 {
        n
    }
}

impl From<MultiPv> for usize {
    fn from(MultiPv(n): MultiPv) -> usize {
        n as usize
    }
}

impl fmt::Display for MultiPv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
