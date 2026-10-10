use crate::error::{GameJoltError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::sync::{Mutex, MutexGuard};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueuedOperation {
    pub endpoint: String,
    pub parameters: Vec<(String, String)>,
}

pub struct OfflineQueue {
    operations: Mutex<Vec<QueuedOperation>>,
    flushing: Mutex<()>,
    capacity: usize,
    persistence_path: Option<PathBuf>,
}

impl OfflineQueue {
    pub fn new(capacity: usize) -> Self {
        Self {
            operations: Mutex::new(Vec::new()),
            flushing: Mutex::new(()),
            capacity,
            persistence_path: None,
        }
    }

    pub fn persistent(capacity: usize, path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let operations = if path.exists() {
            let bytes = std::fs::read(&path)
                .map_err(|error| GameJoltError::QueuePersistenceError(error.to_string()))?;
            bincode::deserialize(&bytes)?
        } else {
            Vec::new()
        };
        Ok(Self {
            operations: Mutex::new(operations),
            flushing: Mutex::new(()),
            capacity,
            persistence_path: Some(path),
        })
    }

    pub async fn enqueue(&self, operation: QueuedOperation) -> Result<()> {
        let mut operations = self.operations.lock().await;
        if operations.len() >= self.capacity {
            return Err(GameJoltError::OfflineQueueFull {
                capacity: self.capacity,
            });
        }
        operations.push(operation);
        self.persist(&operations)
    }

    pub async fn pending(&self) -> usize {
        self.operations.lock().await.len()
    }

    pub(crate) async fn begin_flush(&self) -> MutexGuard<'_, ()> {
        self.flushing.lock().await
    }

    pub(crate) async fn next(&self) -> Option<QueuedOperation> {
        self.operations.lock().await.first().cloned()
    }

    pub(crate) async fn acknowledge(&self) -> Result<()> {
        let mut pending = self.operations.lock().await;
        if pending.is_empty() {
            return Ok(());
        }
        let completed = pending.remove(0);
        if let Err(error) = self.persist(&pending) {
            pending.insert(0, completed);
            return Err(error);
        }
        Ok(())
    }

    fn persist(&self, operations: &[QueuedOperation]) -> Result<()> {
        if let Some(path) = &self.persistence_path {
            let bytes = bincode::serialize(operations)?;
            std::fs::write(path, bytes)
                .map_err(|error| GameJoltError::QueuePersistenceError(error.to_string()))?;
        }
        Ok(())
    }
}
