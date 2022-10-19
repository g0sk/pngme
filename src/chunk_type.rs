use crate::Error;
use std::{fmt, str::FromStr};

#[derive(Debug, Clone)]
pub struct ChunkType {
    chunks: [u8; 4],
}

impl FromStr for ChunkType {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 4 {
            return Err(Box::new(ChunkTypeError::InvalidLength(s.len())));
        }
        let characters = s.as_bytes();
        for character in characters {
            if !Self::is_valid_byte(*character) {
                return Err(Box::new(ChunkTypeError::InvalidCharacter(*character)));
            }
        }

        match <[u8; 4]>::try_from(characters) {
            Ok(res) => Ok(ChunkType { chunks: res }),
            Err(err) => Err(Box::new(err)),
        }
    }
}

impl fmt::Display for ChunkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.chunks))
    }
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = Error;
    fn try_from(bytes: [u8; 4]) -> Result<Self, Self::Error> {
        Ok(ChunkType { chunks: bytes })
    }
}

impl PartialEq for ChunkType {
    fn eq(&self, other: &Self) -> bool {
        self.chunks == other.chunks
    }
}

impl Eq for ChunkType {}

impl ChunkType {
    //Returns the raw bytes contained in this chunk
    pub fn bytes(&self) -> [u8; 4] {
        self.chunks
    }
    //Values need to be in range A-Z (65-90) / a-z (97-122)
    pub fn is_valid(&self) -> bool {
        let bytes = self.bytes();
        for byte in bytes {
            if !Self::is_valid_byte(byte) {
                return false;
            }
        }
        self.is_reserved_bit_valid()
    }

    //Valid bytes are represented by the characters A-Z (65-90) or a-z (97-122)
    pub fn is_valid_byte(byte: u8) -> bool {
        byte.is_ascii_alphabetic()
    }

    /**
     * Ancillary bit: bit 5 of first byte
     * 0 (uppercase) = critical, 1 (lowecase) = ancillary
     * Chunks that are neccesary for successfull display of the file's content are called "critical chunks"
     */
    pub fn is_critical(&self) -> bool {
        match self.chunks[0] >> 5 & 0x1 {
            0 => true,
            _ => false,
        }
    }
    /**
     * Private bit: bit 5 of third byte
     * 0 (uppercase) = public, 1 (lowecase) = private
     */
    pub fn is_public(&self) -> bool {
        match self.chunks[1] >> 5 & 0x1 {
            0 => true,
            _ => false,
        }
    }

    /**
     * Reserved bit: bit 5 of third byte
     * Must be 0 (uppercase) in files conforming to this version of PNG.
     */
    pub fn is_reserved_bit_valid(&self) -> bool {
        match self.chunks[2] >> 5 & 0x1 {
            0 => true,
            _ => false,
        }
    }

    /**
     * Safe-to-copy bit: bit 5 of fourth byte
     * 0 (uppercase) = unsafe to copy, 1 (lowercase) = safe to copy.
     */
    pub fn is_safe_to_copy(&self) -> bool {
        match self.chunks[3] >> 5 & 0x1 {
            1 => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum ChunkTypeError {
    InvalidCharacter(u8),
    InvalidLength(usize),
}

impl std::error::Error for ChunkTypeError {}

impl fmt::Display for ChunkTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCharacter(byte) => write!(f, "invalid character: {}", byte),
            Self::InvalidLength(length) => write!(f, "length must be 4, current: {}", length),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
