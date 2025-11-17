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

    pub fn from_bytes(bytes: &[u8]) -> Result<(&ArchivedEntry, usize), skore_core::Error> {
        let len = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;

        todo!("Lots of reading to do here")
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
