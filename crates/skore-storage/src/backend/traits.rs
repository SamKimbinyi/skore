use skore_core::Result;
#[derive(Debug, Clone, Default)]
pub struct BackendStats {
    pub total_bytes: u64,

    pub live_bytes: u64,

    pub dead_bytes: u64,

    pub waste_ratio: f64,

    pub entry_count: usize,
}

impl BackendStats {
    pub fn new(total_bytes: u64, live_bytes: u64, entry_count: usize) -> BackendStats {
        let dead_bytes = total_bytes.saturating_sub(live_bytes);
        let waste_ratio = if total_bytes > 0 {
            dead_bytes as f64 / live_bytes as f64
        } else {
            0.0
        };
        Self {
            total_bytes,
            live_bytes,
            dead_bytes,
            waste_ratio,
            entry_count,
        }
    }
}

pub trait StorageBackend: Send + Sync {
    fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;

    fn delete(&self, key: &[u8]) -> Result<()>;

    fn flush(&self) -> Result<()>;

    fn compact(&self) -> Result<BackendStats>;
    fn stats(&self) -> Result<BackendStats>;

    fn clear(&self) -> Result<()>;

    fn len(&self) -> Result<usize> {
        Ok(self.stats()?.entry_count)
    }

    fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }
}
