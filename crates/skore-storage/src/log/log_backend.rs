use crate::backend::{BackendStats, StorageBackend};
use crate::entry::Entry;
use memmap2::Mmap;
use skore_core::{Error, LockResultExt, Result};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Copy)]

pub struct EntryPos {
    offset: u64,
    len: usize,
}

pub struct LogBackend {
    path: PathBuf,
    file: Arc<RwLock<File>>,
    mmap: Arc<RwLock<Option<Mmap>>>,
    index: Arc<RwLock<HashMap<Vec<u8>, EntryPos>>>,
    file_size: Arc<RwLock<u64>>,
}

impl LogBackend {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;

        let file_size = file.metadata()?.len();

        let backend = Self {
            path: path.clone(),
            file: Arc::new(RwLock::new(file)),
            mmap: Arc::new(RwLock::new(None)),
            index: Arc::new(RwLock::new(HashMap::<Vec<u8>, EntryPos>::new())),
            file_size: Arc::new(RwLock::new(file_size)),
        };

        if file_size > 0 {
            backend.rebuild_index()?;
        }
        Ok(backend)
    }

    pub fn recover_from_crash(&self, path: &Path) -> Result<()> {
        let backup_path = path.with_extension("old");
        let temp_path = path.with_extension("tmp");

        // Scenario 1: Crash after first rename, before second rename
        if !path.exists() && backup_path.exists() {
            std::fs::rename(&backup_path, path)?;
            eprintln!("Recovered from incomplete compaction (restored backup)");
        }

        // Scenario 2: Crash during compaction, before swap
        if path.exists() && temp_path.exists() {
            std::fs::remove_file(&temp_path)?;
            eprintln!("Cleaned up incomplete compaction temp file");
        }

        // Scenario 3: Successful compaction, backup not deleted
        if path.exists() && backup_path.exists() {
            std::fs::remove_file(&backup_path)?;
        }

        Ok(())
    }

    pub fn rebuild_index(&self) -> Result<()> {
        self.remap()?;

        let mmap_guard = self.mmap.read().poison_err()?;

        let mmap = mmap_guard
            .as_ref()
            .ok_or_else(|| Error::internal("No mmap available"))?;

        let mut index = self.index.write().poison_err()?;

        let mut offset = 0;

        let bytes = &mmap[..];

        while offset < bytes.len() {
            let start_offset = offset;

            match Entry::from_bytes(&bytes[offset..]) {
                Ok((archived_entry, len)) => {
                    let key = archived_entry.key.to_vec();

                    if archived_entry.value.is_some() {
                        index.insert(
                            key,
                            EntryPos {
                                offset: start_offset as u64,
                                len,
                            },
                        );
                    } else {
                        index.remove(&key);
                    }
                    offset += len;
                }
                Err(_) => {
                    break;
                }
            }
        }

        //update the filesize
        *self.file_size.write().unwrap() = offset as u64;

        Ok(())
    }

    fn remap(&self) -> Result<()> {
        let file = self.file.read().poison_err()?;
        let file_size = *self
            .file_size
            .read()
            .map_err(|e| Error::internal(format!("Poisoned: {}", e)))?;

        let mut mmap_guard = self.mmap.write().poison_err()?;

        if file_size > 0 {
            let new_mmap = unsafe { memmap2::Mmap::map(&*file)? };
            *mmap_guard = Some(new_mmap);
        }
        Ok(())
    }

    pub fn append_entry(&self, entry: Entry) -> Result<()> {
        let bytes = entry.to_bytes();
        let entry_len = bytes.len();

        let mut file = self.file.write().poison_err()?;

        let mut file_size = self.file_size.write().poison_err()?;

        let offset = *file_size;

        //Go to the end of the file
        file.seek(SeekFrom::Start(offset))?;

        file.write_all(&bytes)?;
        file.flush()?;
        *file_size += entry_len as u64;

        let mut index = self.index.write().poison_err()?;

        if entry.value.is_some() {
            index.insert(
                entry.key.clone(),
                EntryPos {
                    offset,
                    len: entry_len as usize,
                },
            );
        } else {
            //tombstone
            index.remove(&entry.key);
        }

        drop(file);
        drop(file_size);
        drop(index);
        self.remap()?;
        Ok(())
    }

    pub fn read_entry(&self, pos: EntryPos) -> Result<Vec<u8>> {
        let mmap_guard = self.mmap.read().poison_err()?;

        let mmap = mmap_guard
            .as_ref()
            .ok_or_else(|| Error::internal("No mmap available"))?;

        let start = pos.offset as usize;
        let end = start + pos.len;

        if end > mmap.len() {
            return Err(Error::corruption("Entry position out of bounds"));
        }

        let (archived_entry, _) = Entry::from_bytes(&mmap[start..end])?;

        match &archived_entry.value {
            Some(archived_entry) => Ok(archived_entry.clone()),
            None => Err(Error::internal("Entry is tombstone")),
        }
    }
}

impl StorageBackend for LogBackend {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let index = self.index.read().poison_err()?;

        if let Some(&pos) = index.get(key) {
            Ok(Some(self.read_entry(pos)?))
        } else {
            Ok(None)
        }
    }

    fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        let new_entry = Entry::new(key, value);
        self.append_entry(new_entry)
    }

    fn delete(&self, key: &[u8]) -> Result<()> {
        let new_entry = Entry::tombstone(key.to_vec());
        self.append_entry(new_entry)
    }

    fn clear(&self) -> Result<()> {
        {
            let mut index = self.index.write().poison_err()?;
            index.clear();
        }

        {
            let file = self.file.write().poison_err()?;
            file.set_len(0)?;
            file.sync_all()?;
        }
        {
            let mut file_size = self.file_size.write().poison_err()?;
            *file_size = 0;
        }
        {
            let mut mmap_guard = self.mmap.write().poison_err()?;
            *mmap_guard = None;
        }
        Ok(())
    }

    fn len(&self) -> Result<usize> {
        Ok(self.index.read().unwrap().len())
    }

    fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    fn compact(&self) -> Result<BackendStats> {
        self.stats()
    }

    fn flush(&self) -> Result<()> {
        let file = self.file.write().poison_err()?;
        file.sync_all()?;
        Ok(())
    }

    fn stats(&self) -> Result<BackendStats> {
        let file_size = *self.file_size.read().poison_err()?;
        let index = self.index.read().poison_err()?;

        let mut live_bytes = 0u64;
        for pos in index.values() {
            live_bytes += pos.len as u64;
        }

        Ok(BackendStats::new(file_size, live_bytes, index.len()))
    }
}

impl Clone for LogBackend {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            file: Arc::clone(&self.file),
            mmap: Arc::clone(&self.mmap),
            index: Arc::clone(&self.index),
            file_size: Arc::clone(&self.file_size),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn temp_store() -> (LogBackend, TempDir) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.db");
        let store = LogBackend::open(&path).unwrap();
        (store, dir)
    }

    #[test]
    fn test_basic_operations() {
        let (store, _dir) = temp_store();

        // Set
        store.put(b"key".to_vec(), b"value".to_vec()).unwrap();

        // Get
        assert_eq!(store.get(b"key").unwrap(), Some(b"value".to_vec()));

        // Delete
        store.delete(b"key").unwrap();
        assert_eq!(store.get(b"key").unwrap(), None);
    }

    #[test]
    fn test_persistence() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.db");

        // Write data
        {
            let store = LogBackend::open(&path).unwrap();
            store.put(b"key1".to_vec(), b"value1".to_vec()).unwrap();
            store.put(b"key2".to_vec(), b"value2".to_vec()).unwrap();
        }

        // Reopen and verify
        {
            let store = LogBackend::open(&path).unwrap();
            assert_eq!(store.get(b"key1").unwrap(), Some(b"value1".to_vec()));
            assert_eq!(store.get(b"key2").unwrap(), Some(b"value2".to_vec()));
        }
    }

    #[test]
    fn test_overwrite() {
        let (store, _dir) = temp_store();

        store.put(b"key".to_vec(), b"value1".to_vec()).unwrap();
        store.put(b"key".to_vec(), b"value2".to_vec()).unwrap();

        assert_eq!(store.get(b"key").unwrap(), Some(b"value2".to_vec()));

        // Should have 1 entry in index (not 2)

        assert_eq!(store.len().unwrap(), 1);
    }

    #[test]
    fn test_multiple_keys() {
        let (store, _dir) = temp_store();

        for i in 0..100 {
            let key = format!("key{}", i).into_bytes();
            let value = format!("value{}", i).into_bytes();
            store.put(key, value).unwrap();
        }

        assert_eq!(store.len().unwrap(), 100);

        for i in 0..100 {
            let key = format!("key{}", i).into_bytes();
            let expected = format!("value{}", i).into_bytes();
            assert_eq!(store.get(&key).unwrap(), Some(expected));
        }
    }
}
