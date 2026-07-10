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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::LogEntry;

    fn make_entry() -> LogEntry {
        LogEntry::new("test", "GET", "/api/test", 200, 42, 10, 200, None)
    }

    #[tokio::test]
    async fn new_logger_creates_parent_directory() {
        let dir = std::env::temp_dir().join(format!("api_logger_test_{}", uuid::Uuid::new_v4()));
        let csv = dir.join("sub/logs.csv");
        let logger = ApiLogger::new(&csv);
        logger.log(&make_entry()).await.unwrap();
        assert!(csv.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn log_writes_entry_to_csv() {
        let dir = std::env::temp_dir().join(format!("api_logger_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let csv = dir.join("logs.csv");
        let logger = ApiLogger::new(&csv);
        logger.log(&make_entry()).await.unwrap();

        let content = std::fs::read_to_string(&csv).unwrap();
        assert!(content.contains("/api/test"));
        assert!(content.contains("GET"));
        assert!(content.contains("200"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn log_appends_multiple_entries() {
        let dir = std::env::temp_dir().join(format!("api_logger_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let csv = dir.join("logs.csv");
        let logger = ApiLogger::new(&csv);

        logger.log(&make_entry()).await.unwrap();
        logger.log(&make_entry()).await.unwrap();
        logger.log(&make_entry()).await.unwrap();

        let content = std::fs::read_to_string(&csv).unwrap();
        assert_eq!(content.lines().count(), 4); // header + 3 entries
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn log_handles_error_entry() {
        let dir = std::env::temp_dir().join(format!("api_logger_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let csv = dir.join("logs.csv");
        let logger = ApiLogger::new(&csv);

        let err_entry = LogEntry::new(
            "test",
            "POST",
            "/api/fail",
            500,
            999,
            10,
            0,
            Some("timeout".into()),
        );
        logger.log(&err_entry).await.unwrap();

        let content = std::fs::read_to_string(&csv).unwrap();
        assert!(content.contains("500"));
        assert!(content.contains("timeout"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn path_returns_configured_path() {
        let logger = ApiLogger::new("some/path.csv");
        assert_eq!(logger.path().to_string_lossy(), "some/path.csv");
    }

    #[test]
    fn new_logger_does_not_create_file_immediately() {
        let dir = std::env::temp_dir().join(format!("api_logger_test_{}", uuid::Uuid::new_v4()));
        let csv = dir.join("nosuch.csv");
        let logger = ApiLogger::new(&csv);
        assert_eq!(logger.path().to_string_lossy(), csv.to_string_lossy());
        assert!(!csv.exists());
    }
}
