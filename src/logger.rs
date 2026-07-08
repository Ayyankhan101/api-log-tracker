use crate::models::LogEntry;
use anyhow::Result;
use csv::WriterBuilder;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Thread-safe, append-only CSV logger. Clone it freely — it's an Arc internally.
#[derive(Clone)]
pub struct ApiLogger {
    path: PathBuf,
    lock: Arc<Mutex<()>>,
}

impl ApiLogger {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        Self {
            path,
            lock: Arc::new(Mutex::new(())),
        }
    }

    /// Append one entry to the CSV file. Writes the header only if the file
    /// doesn't exist yet. Safe to call concurrently from many tasks.
    pub async fn log(&self, entry: &LogEntry) -> Result<()> {
        let _guard = self.lock.lock().await;
        let path = self.path.clone();
        let entry = entry.clone();

        // csv::Writer is sync, so do the actual file I/O on a blocking thread.
        tokio::task::spawn_blocking(move || -> Result<()> {
            let file_exists = path.exists();
            let file = OpenOptions::new().create(true).append(true).open(&path)?;
            let mut wtr = WriterBuilder::new()
                .has_headers(!file_exists)
                .from_writer(file);
            wtr.serialize(&entry)?;
            wtr.flush()?;
            Ok(())
        })
        .await??;

        Ok(())
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}
