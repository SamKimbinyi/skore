use memmap2::Mmap;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
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
