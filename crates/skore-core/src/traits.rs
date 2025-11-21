use crate::{Error, Result};
use std::sync::LockResult;
pub trait LockResultExt<T> {
    fn poison_err(self) -> Result<T>;
}

impl<T> LockResultExt<T> for LockResult<T> {
    fn poison_err(self) -> Result<T> {
        self.map_err(|e| Error::internal(format!("Lock poisoned: {}", e)))
    }
}
