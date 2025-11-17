use rkyv::util::AlignedVec;
use rkyv::{Archive, Deserialize, Serialize, rancor::Error, with::AsBox};
use std::fmt::Debug;
use std::time::{SystemTime, UNIX_EPOCH};

///Disk format, going to attempt rkyv and memmap
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct Entry {
    pub timestamp: u64,
    #[rkyv(with = AsBox)]
    pub key: Vec<u8>,
    pub value: Option<Vec<u8>>,
}

impl Entry {
    pub fn new(key: Vec<u8>, value: Vec<u8>) -> Self {
        Entry {
            timestamp: current_timestamp(),
            key,
            value: Some(value),
        }
    }

    pub fn tombstone(key: Vec<u8>) -> Self {
        Entry {
            timestamp: current_timestamp(),
            key,
            value: None,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let archived = rkyv::to_bytes::<Error>(self).expect("Failed to serialize entry");
        let mut result = Vec::with_capacity(4 + archived.len());
        result.extend_from_slice(&(archived.len() as u32).to_le_bytes());
        result.extend_from_slice(&archived);
        result
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Entry, usize), skore_core::Error> {
        let len = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;

        //We have to use an aligned vector because the first 4 bytes are used for size
        let mut aligned = AlignedVec::<16>::new();
        aligned.extend_from_slice(&bytes[4..4 + len]);

        let archived =
            rkyv::access::<ArchivedEntry, Error>(&aligned).expect("Failed to load ArchivedEntry");
        let data: Entry =
            rkyv::deserialize::<Entry, Error>(archived).expect("Failed to deserialize entry");
        Ok((data, 4 + len))
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_entry_creation() {
        let key = b"test_key".to_vec();
        let value = b"test_value".to_vec();
        let entry = Entry::new(key.clone(), value.clone());

        assert_eq!(entry.key, key);
        assert_eq!(entry.value, Some(value));
        assert!(entry.timestamp > 0);
    }

    #[test]
    fn test_tombstone_creation() {
        let key = b"deleted_key".to_vec();
        let entry = Entry::tombstone(key.clone());

        assert_eq!(entry.key, key);
        assert_eq!(entry.value, None);
        assert!(entry.timestamp > 0);
    }

    #[test]
    fn test_timestamp_increments() {
        let entry1 = Entry::new(b"key1".to_vec(), b"value1".to_vec());
        std::thread::sleep(std::time::Duration::from_secs(1));
        let entry2 = Entry::new(b"key2".to_vec(), b"value2".to_vec());

        assert!(entry2.timestamp >= entry1.timestamp);
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let key = b"test_key".to_vec();
        let value = b"test_value".to_vec();
        let entry = Entry::new(key.clone(), value.clone());

        let bytes = entry.to_bytes();
        let (deserialized, size) = Entry::from_bytes(&bytes).expect("Failed to deserialize");

        assert_eq!(deserialized.key, entry.key);
        assert_eq!(deserialized.value, entry.value);
        assert_eq!(deserialized.timestamp, entry.timestamp);
        assert_eq!(size, bytes.len());
    }

    #[test]
    fn test_tombstone_serialize_deserialize() {
        let key = b"deleted_key".to_vec();
        let entry = Entry::tombstone(key.clone());

        let bytes = entry.to_bytes();
        let (deserialized, size) = Entry::from_bytes(&bytes).expect("Failed to deserialize");

        assert_eq!(deserialized.key, entry.key);
        assert_eq!(deserialized.value, None);
        assert_eq!(deserialized.timestamp, entry.timestamp);
        assert_eq!(size, bytes.len());
    }
}
