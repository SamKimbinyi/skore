use skore_core::{Result, Store};
use std::collections::BTreeMap;

pub struct MemoryStore {
    data: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        MemoryStore {
            data: BTreeMap::new(),
        }
    }
}

impl Store for MemoryStore {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.data.get(key).cloned())
    }

    fn set(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.data.insert(key, value);
        Ok(())
    }

    fn delete(&mut self, key: &[u8]) -> Result<()> {
        self.data.remove(key);
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        self.data.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get() {
        let mut store = MemoryStore::new();
        store.set(b"key".to_vec(), b"value".to_vec()).unwrap();
        assert_eq!(store.get(b"key").unwrap(), Some(b"value".to_vec()));
    }

    #[test]
    fn test_get_nonexistent_key() {
        let store = MemoryStore::new();
        assert_eq!(store.get(b"missing").unwrap(), None);
    }

    #[test]
    fn test_overwrite_value() {
        let mut store = MemoryStore::new();
        store.set(b"key".to_vec(), b"value1".to_vec()).unwrap();
        store.set(b"key".to_vec(), b"value2".to_vec()).unwrap();
        assert_eq!(store.get(b"key").unwrap(), Some(b"value2".to_vec()));
    }

    #[test]
    fn test_delete() {
        let mut store = MemoryStore::new();
        store.set(b"key".to_vec(), b"value".to_vec()).unwrap();
        store.delete(b"key").unwrap();
        assert_eq!(store.get(b"key").unwrap(), None);
    }

    #[test]
    fn test_delete_nonexistent_key() {
        let mut store = MemoryStore::new();
        // Deleting a key that doesn't exist should not panic
        assert!(store.delete(b"missing").is_ok());
    }

    #[test]
    fn test_flush() {
        let mut store = MemoryStore::new();
        store.set(b"key1".to_vec(), b"value1".to_vec()).unwrap();
        store.set(b"key2".to_vec(), b"value2".to_vec()).unwrap();
        store.flush().unwrap();
        assert_eq!(store.get(b"key1").unwrap(), None);
        assert_eq!(store.get(b"key2").unwrap(), None);
    }

    #[test]
    fn test_multiple_keys() {
        let mut store = MemoryStore::new();
        store.set(b"a".to_vec(), b"1".to_vec()).unwrap();
        store.set(b"b".to_vec(), b"2".to_vec()).unwrap();
        store.set(b"c".to_vec(), b"3".to_vec()).unwrap();

        assert_eq!(store.get(b"a").unwrap(), Some(b"1".to_vec()));
        assert_eq!(store.get(b"b").unwrap(), Some(b"2".to_vec()));
        assert_eq!(store.get(b"c").unwrap(), Some(b"3".to_vec()));
    }

    #[test]
    fn test_empty_store() {
        let store = MemoryStore::new();
        // Empty store should return None for any key
        assert_eq!(store.get(b"any").unwrap(), None);
    }
}
