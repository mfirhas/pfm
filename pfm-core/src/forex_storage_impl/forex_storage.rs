// implementations for database to store forex data polled from the APIs.
// using filesystem with tokio

use crate::global::StorageFS;

pub(crate) struct ForexStorageImpl {
    fs: StorageFS,
}

impl ForexStorageImpl {
    pub(crate) fn new(fs: StorageFS) -> Self {
        Self { fs }
    }
}
