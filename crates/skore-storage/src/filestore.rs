use crate::entry::Entry;
use memmap2::Mmap;
use skore_core::{Error, Result, Store};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Copy)]

struct EntryPos {
    offset: u64,
    len: usize,
}

pub struct FileStore {
    path: PathBuf,
    file: Arc<RwLock<File>>,
    mmap: Arc<RwLock<Option<Mmap>>>,
    index: Arc<RwLock<HashMap<Vec<u8>, EntryPos>>>,
    file_size: Arc<RwLock<u64>>,
}

impl FileStore {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;

        let file_size = file.metadata()?.len();

        let store = Self {
            path: path.clone(),
            file: Arc::new(RwLock::new(file)),
            mmap: Arc::new(RwLock::new(None)),
            index: Arc::new(RwLock::new(HashMap::<Vec<u8>, EntryPos>::new())),
            file_size: Arc::new(RwLock::new(file_size)),
        };

        if file_size > 0 {
            store.rebuild_index()?;
        }
        Ok(store)
    }

    pub fn rebuild_index(&self) -> Result<()> {
        self.remap()?;

        let mmap_guard = self
            .mmap
            .read()
            .map_err(|e| Error::internal(format!("Lock poisoned: {}", e)))?;

        let mmap = mmap_guard
            .as_ref()
            .ok_or_else(|| Error::internal("No mmap available"))?;

        let mut index = self
            .index
            .write()
            .map_err(|e| Error::internal(format!("Lock poisoned: {}", e)))?;

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
        let file = self
            .file
            .read()
            .map_err(|e| Error::internal(format!("Lock poisoned: {}", e)))?;
        let file_size = *self
            .file_size
            .read()
            .map_err(|e| Error::internal(format!("Poisoned: {}", e)))?;

        let mut mmap_guard = self
            .mmap
            .write()
            .map_err(|e| Error::internal(format!("Lock poisoned: {}", e)))?;

        if file_size > 0 {
            let new_mmap = unsafe { memmap2::Mmap::map(&*file)? };
            *mmap_guard = Some(new_mmap);
        }
        Ok(())
    }

    pub fn append_entry(&self, entry: Entry) -> Result<()> {
        let bytes = entry.to_bytes();
        let entry_len = bytes.len();

        let mut file = self
            .file
            .write()
            .map_err(|e| Error::internal(format!("Lock poisoned: {}", e)))?;

        let mut file_size = self
            .file_size
            .write()
            .map_err(|e| Error::internal(format!("Lock Poisoned: {}", e)))?;

        let offset = *file_size;

        //Go to the end of the file
        file.seek(SeekFrom::Start(offset))?;

        file.write_all(&bytes)?;
        file.flush()?;
        *file_size += entry_len as u64;

        let mut index = self
            .index
            .write()
            .map_err(|e| Error::internal(format!("Lock Poisoned: {}", e)))?;

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
            index.remove(&bytes);
        }

        drop(file);
        drop(file_size);
        drop(index);
        self.remap()?;
        Ok(())
    }

    pub fn read_entry(&self, pos: EntryPos) -> Result<Vec<u8>> {
        todo!("Read entry")
    }

    pub fn len(&self) -> usize {
        todo!("Get number of keys in store? Maybe put in trait")
    }

    pub fn is_empty(&self) -> bool {
        todo!("Get the size of keys in store? Maybe put in trait")
    }
}

impl Store for FileStore {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        todo!()
    }

    fn set(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        todo!()
    }

    fn delete(&self, key: &[u8]) -> Result<()> {
        todo!()
    }

    fn flush(&self) -> Result<()> {
        todo!()
    }
}

impl Clone for FileStore {
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
