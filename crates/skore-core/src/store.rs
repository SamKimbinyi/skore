pub trait Store: Send + Sync {
    fn get(&self, key: &[u8]) -> Result<Vec<u8>>;
    fn set(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
    fn delete(&mut self,key: &[u8]) -> Result<()>;
    fn flush(&mut self) -> Result<()>;
}