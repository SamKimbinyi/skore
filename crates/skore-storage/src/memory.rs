use skore_core::{LockResultExt, Result, Store};
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

pub struct MemoryStore {
    data: Arc<RwLock<BTreeMap<Vec<u8>, Vec<u8>>>>,
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStore {
    pub fn new() -> Self {
        MemoryStore {
            data: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }
}

impl Store for MemoryStore {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let data = self.data.read().poison_err()?;

        Ok(data.get(key).cloned())
    }

    fn set(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        let mut data = self.data.write().poison_err()?;

        data.insert(key, value);
        Ok(())
    }

    fn delete(&self, key: &[u8]) -> Result<()> {
        let mut data = self.data.write().poison_err()?;

        data.remove(key);
        Ok(())
    }

    fn clear(&self) -> Result<()> {
        let mut data = self.data.write().poison_err()?;

        data.clear();
        Ok(())
    }
    fn len(&self) -> Result<usize> {
        let data = self.data.read().poison_err()?;

        Ok(data.len())
    }

    fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get() {
        let store = MemoryStore::new();
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
        let store = MemoryStore::new();
        store.set(b"key".to_vec(), b"value1".to_vec()).unwrap();
        store.set(b"key".to_vec(), b"value2".to_vec()).unwrap();
        assert_eq!(store.get(b"key").unwrap(), Some(b"value2".to_vec()));
    }

    #[test]
    fn test_delete() {
        let store = MemoryStore::new();
        store.set(b"key".to_vec(), b"value".to_vec()).unwrap();
        store.delete(b"key").unwrap();
        assert_eq!(store.get(b"key").unwrap(), None);
    }

    #[test]
    fn test_delete_nonexistent_key() {
        let store = MemoryStore::new();
        // Deleting a key that doesn't exist should not panic
        assert!(store.delete(b"missing").is_ok());
    }

    #[test]
    fn test_multiple_keys() {
        let store = MemoryStore::new();
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
        assert_eq!(store.get(b"any").unwrap(), None);
    }
}
