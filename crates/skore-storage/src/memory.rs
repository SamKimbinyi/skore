use skore_core::{Result, Store};

pub struct MemoryStore;

impl Store for MemoryStore {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        todo!("implement this")
    }

    fn set(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        todo!("implement this")
    }

    fn delete(&mut self, key: &[u8]) -> Result<()> {
        todo!("implement this")
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}
