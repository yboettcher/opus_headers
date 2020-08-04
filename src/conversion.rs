//! small wrappers around standard functions
//! Note that this could panic if used incorrectly
//! Any caller will have to make sure the supplied slice has at least the required amount of entries

/// needs two entries
pub(crate) fn to_u16(bytes: &[u8]) -> u16 {
    u16::from_le_bytes([bytes[0], bytes[1]])
}

/// needs four entries
pub(crate) fn to_u32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

/// needs two entries
pub(crate) fn to_i16(bytes: &[u8]) -> i16 {
    i16::from_le_bytes([bytes[0], bytes[1]])
} 
