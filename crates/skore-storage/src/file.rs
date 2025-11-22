use crate::backend::{BackendStats, StorageBackend};
use crate::log::LogBackend;
use skore_core::{Result, Store};
use std::path::Path;
use std::sync::Arc;

#[derive(Clone)]
pub struct FileStore {
    backend: Arc<dyn StorageBackend>,
}

impl FileStore {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let backend = LogBackend::open(path)?;
        Ok(Self {
            backend: Arc::new(backend),
        })
    }
    pub fn with_backend(backend: Arc<dyn StorageBackend>) -> Self {
        Self { backend }
    }

    pub fn stats(&self) -> Result<BackendStats> {
        self.backend.stats()
    }

    pub fn compact(&self) -> Result<BackendStats> {
        self.backend.compact()
    }
}

impl Store for FileStore {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.backend.get(key)
    }

    fn set(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.backend.put(key, value)
    }

    fn delete(&self, key: &[u8]) -> Result<()> {
        self.backend.delete(key)
    }

    fn clear(&self) -> Result<()> {
        self.backend.clear()
    }

    fn len(&self) -> Result<usize> {
        self.backend.len()
    }

    fn is_empty(&self) -> Result<bool> {
        self.backend.is_empty()
    }
}
