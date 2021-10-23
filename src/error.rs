use thiserror::Error;

use std::io;
use block_modes;
use base64;

#[derive(Error, Debug)]
pub enum RncmError {
    #[error("IOError")]
    IOError(#[from] io::Error),
    #[error("InvalidKeyIvLength")]
    InvalidKeyIvLength(#[from] block_modes::InvalidKeyIvLength),
    #[error("BlockModeError")]
    BlockModeError(#[from] block_modes::BlockModeError),
    #[error("Base64De")]
    Base64DecoderError(#[from] base64::DecodeError)
}

