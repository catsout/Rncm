use thiserror::Error;

use block_modes;

#[derive(Error, Debug)]
pub enum AESDecodeError {
    #[error("InvalidKeyIvLength")]
    InvalidKeyIvLength(#[from] block_modes::InvalidKeyIvLength),
    #[error("BlockModeError")]
    BlockModeError(#[from] block_modes::BlockModeError)
}

