//! A private prelude module.

pub(crate) use crate::bytes::FromBytes;
pub(crate) use frombytes::{Bytes, Error, Result};

pub(crate) use crate::util::BoolExt;

#[cfg(test)]
pub(crate) use crate::util::fromhex::FromHex;
