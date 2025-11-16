use skore_core::{Result, Store};
use skore_storage::MemoryStore;

pub struct Skore {
    inner: Box<dyn Store>,
}

impl Default for Skore {
    fn default() -> Self {
        Skore {
            inner: Box::new(MemoryStore::new()),
        }
    }
}

impl Skore {
    pub fn new<S: Store + 'static>(store: S) -> Self {
        Skore {
            inner: Box::new(store),
        }
    }

    pub fn get(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.inner.get(key)
    }
    pub fn set(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self.inner.set(key.to_vec(), value.to_vec())
    }
    pub fn delete(&mut self, key: &[u8]) -> Result<()> {
        self.inner.delete(key)
    }
    pub fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skore_storage::MemoryStore;

    #[test]
    fn test_default_constructor() {
        let mut skore = Skore::default();
        let key = b"foo";
        let value = b"bar";

        // Initially key should not exist
        assert!(skore.get(key).unwrap().is_none());

        // Set a value
        skore.set(key, value).unwrap();
        assert_eq!(skore.get(key).unwrap(), Some(value.to_vec()));

        // Delete the value
        skore.delete(key).unwrap();
        assert!(skore.get(key).unwrap().is_none());

        // Flush should succeed (no-op for MemoryStore)
        skore.flush().unwrap();
    }

    #[test]
    fn test_new_with_custom_store() {
        let custom_store = MemoryStore::new();
        let mut skore = Skore::new(custom_store);

        let key = b"hello";
        let value = b"world";

        // Set and get
        skore.set(key, value).unwrap();
        let stored = skore.get(key).unwrap();
        assert_eq!(stored, Some(value.to_vec()));

        // Delete
        skore.delete(key).unwrap();
        assert!(skore.get(key).unwrap().is_none());

        // Flush
        skore.flush().unwrap();
    }

    #[test]
    fn test_overwrite_value() {
        let mut skore = Skore::default();
        let key = b"key";
        let value1 = b"value1";
        let value2 = b"value2";

        skore.set(key, value1).unwrap();
        assert_eq!(skore.get(key).unwrap(), Some(value1.to_vec()));

        // Overwrite
        skore.set(key, value2).unwrap();
        assert_eq!(skore.get(key).unwrap(), Some(value2.to_vec()));
    }

    #[test]
    fn test_multiple_keys() {
        let mut skore = Skore::default();
        let keys: &[&[u8]] = &[b"one", b"two", b"three"];
        let values: &[&[u8]] = &[b"1", b"2", b"3"];

        for (k, v) in keys.iter().zip(values.iter()) {
            skore.set(k, v).unwrap();
        }

        for (k, v) in keys.iter().zip(values.iter()) {
            assert_eq!(skore.get(k).unwrap(), Some(v.to_vec()));
        }

        // Delete one key
        skore.delete(b"two").unwrap();
        assert!(skore.get(b"two").unwrap().is_none());
    }

    #[test]
    fn test_multiple_vector_keys() {
        let mut skore = Skore::default();

        let keys: Vec<Vec<u8>> = vec![b"one".to_vec(), b"two".to_vec(), b"three".to_vec()];
        let values: Vec<Vec<u8>> = vec![b"1".to_vec(), b"2".to_vec(), b"3".to_vec()];

        for (k, v) in keys.iter().zip(values.iter()) {
            skore.set(k, v).unwrap();
        }

        for (k, v) in keys.iter().zip(values.iter()) {
            assert_eq!(skore.get(k).unwrap(), Some(v.clone()));
        }
        skore.delete(b"two").unwrap();
        assert!(skore.get(b"two").unwrap().is_none());
    }
}
