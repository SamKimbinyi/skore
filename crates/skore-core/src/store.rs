use crate::error::Result;
pub trait Store: Send + Sync {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn set(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
    fn delete(&self, key: &[u8]) -> Result<()>;
    fn clear(&self) -> Result<()>;
    fn len(&self) -> Result<usize>;
    fn is_empty(&self) -> Result<bool>;
}
