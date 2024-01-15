use super::types::*;

pub const SEPARATOR_LENGTH: usize = 4;
pub const ENCRYPTED_FILE_VERSION: u8 = 1;
pub const HEADER_RESERVE_LENGTH: usize = 4;
pub const SEPARATOR: [u8; SEPARATOR_LENGTH] = [0u8; SEPARATOR_LENGTH];
pub const DEFAULT_CHUNK_SIZE: u64 = 5 * 1024 * 1024;