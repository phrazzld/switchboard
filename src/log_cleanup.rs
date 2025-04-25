//! Log cleanup and management functionality
//!
//! This module provides utilities for cleaning up old log files based on age,
//! helping to manage disk space usage in development environments.
//!
//! Key features:
//! - Automatic cleanup of logs older than a configurable age threshold
//! - Handles both app and test log directories
//! - Can be triggered either at startup or via CLI flag
//! - Provides detailed reporting on what files were cleaned up

use crate::config::Config;
use crate::logger::{APP_LOG_SUBDIR, DEFAULT_LOG_DIR, TEST_LOG_SUBDIR};
use chrono::{DateTime, Duration, Local};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Results from the log cleanup operation
pub struct CleanupResult {
    /// Number of files that were successfully deleted
    pub files_removed: usize,
    /// Total size of files removed (in bytes)
    pub bytes_removed: u64,
    /// Any files that couldn't be removed (with reasons)
    pub failed_files: Vec<(PathBuf, String)>,
}

impl Default for CleanupResult {
    fn default() -> Self {
        CleanupResult {
            files_removed: 0,
            bytes_removed: 0,
            failed_files: Vec::new(),
        }
    }
}

impl CleanupResult {
    /// Creates a new empty cleanup result
    pub fn new() -> Self {
        Self::default()
    }

    /// Merges another cleanup result into this one
    pub fn merge(&mut self, other: CleanupResult) {
        self.files_removed += other.files_removed;
        self.bytes_removed += other.bytes_removed;
        self.failed_files.extend(other.failed_files);
    }
}

/// Performs log cleanup based on configuration
///
/// This function deletes log files older than the specified max age from both
/// the application and test log directories. It's safe to call even if the
/// log max age is not set in the configuration (it will do nothing in that case).
///
/// The cleanup process:
/// 1. Checks if log_max_age_days is configured and > 0
/// 2. Scans both app and test log directories
/// 3. Deletes files older than the specified threshold
/// 4. Reports results via the returned CleanupResult
///
/// # Arguments
/// * `config` - The application configuration containing log_max_age_days
///
/// # Returns
/// A CleanupResult containing statistics about the cleanup operation
///
/// # Examples
/// ```no_run
/// use switchboard::config::Config;
/// use switchboard::log_cleanup::cleanup_logs;
///
/// // Create a configuration with a max log age of 7 days
/// let mut config = Config::default();
/// config.log_max_age_days = Some(7);
///
/// // Perform the cleanup
/// let result = cleanup_logs(&config);
/// println!("Removed {} files ({} bytes)", result.files_removed, result.bytes_removed);
/// ```
pub fn cleanup_logs(config: &Config) -> CleanupResult {
    // Check if log_max_age_days is configured
    let max_age_days = match config.log_max_age_days {
        Some(days) if days > 0 => days,
        _ => {
            debug!("Log cleanup skipped - max age not configured or set to zero");
            return CleanupResult::new();
        }
    };

    info!(max_age_days, "Starting log cleanup");

    // Initialize the cleanup result
    let mut result = CleanupResult::new();

    // Clean up app logs
    let app_dir = PathBuf::from(DEFAULT_LOG_DIR).join(APP_LOG_SUBDIR);
    if app_dir.exists() {
        let app_result = cleanup_directory(&app_dir, max_age_days);
        info!(
            directory = %app_dir.display(),
            files_removed = app_result.files_removed,
            bytes_removed = app_result.bytes_removed,
            "Cleaned up app logs"
        );
        result.merge(app_result);
    }

    // Clean up test logs
    let test_dir = PathBuf::from(DEFAULT_LOG_DIR).join(TEST_LOG_SUBDIR);
    if test_dir.exists() {
        let test_result = cleanup_directory(&test_dir, max_age_days);
        info!(
            directory = %test_dir.display(),
            files_removed = test_result.files_removed,
            bytes_removed = test_result.bytes_removed,
            "Cleaned up test logs"
        );
        result.merge(test_result);
    }

    // Report any failures
    if !result.failed_files.is_empty() {
        warn!(
            count = result.failed_files.len(),
            "Some files could not be removed during log cleanup"
        );
        for (path, reason) in &result.failed_files {
            warn!(path = %path.display(), reason, "Failed to remove log file");
        }
    }

    info!(
        files_removed = result.files_removed,
        bytes_removed = result.bytes_removed,
        "Log cleanup completed"
    );

    result
}

/// Cleans up log files in a specific directory that are older than the max age
///
/// This function removes log files in the specified directory that are older than
/// the given maximum age in days. It returns a CleanupResult with statistics about
/// the operation.
///
/// # Arguments
/// * `directory` - Path to the directory to clean up
/// * `max_age_days` - Maximum age of files in days before deletion
///
/// # Returns
/// A CleanupResult with details about the cleanup operation
fn cleanup_directory(directory: &Path, max_age_days: u32) -> CleanupResult {
    let mut result = CleanupResult::new();
    let cutoff_date = Local::now() - Duration::days(max_age_days as i64);

    debug!(directory = %directory.display(), max_age_days, "Scanning directory for old log files");

    // Read the directory
    let dir_entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(e) => {
            warn!(directory = %directory.display(), error = %e, "Failed to read directory for cleanup");
            return result;
        }
    };

    // Process each entry
    for entry in dir_entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                warn!(error = %e, "Failed to read directory entry during cleanup");
                continue;
            }
        };

        let path = entry.path();

        // Skip directories and non-log files
        if path.is_dir() || !is_log_file(&path) {
            continue;
        }

        // Get file metadata
        let metadata = match fs::metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                result
                    .failed_files
                    .push((path.clone(), format!("Failed to read metadata: {}", e)));
                continue;
            }
        };

        // Get file modification time
        let modified = match metadata.modified() {
            Ok(time) => DateTime::<Local>::from(time).with_timezone(&Local),
            Err(e) => {
                result.failed_files.push((
                    path.clone(),
                    format!("Failed to get modification time: {}", e),
                ));
                continue;
            }
        };

        // Check if file is older than the cutoff date
        if modified < cutoff_date {
            debug!(path = %path.display(), modified = %modified, "Removing old log file");

            // Try to remove the file
            match fs::remove_file(&path) {
                Ok(_) => {
                    result.files_removed += 1;
                    result.bytes_removed += metadata.len();
                }
                Err(e) => {
                    result
                        .failed_files
                        .push((path.clone(), format!("Failed to remove file: {}", e)));
                }
            }
        }
    }

    result
}

/// Checks if a path is a log file based on its extension
///
/// This function determines if a file is a log file by checking its extension.
/// It supports both .log files and .log.YYYY-MM-DD format (for rotated logs).
///
/// # Arguments
/// * `path` - The path to check
///
/// # Returns
/// true if the file appears to be a log file, false otherwise
fn is_log_file(path: &Path) -> bool {
    if let Some(file_name) = path.file_name() {
        let file_name = file_name.to_string_lossy();

        // Check for simple .log extension
        if file_name.ends_with(".log") {
            return true;
        }

        // Check for rotated log files with date suffix (.log.YYYY-MM-DD)
        if let Some(idx) = file_name.find(".log.") {
            // Make sure what follows .log. looks like a date (all remaining chars are digits, dashes, or underscores)
            let suffix = &file_name[(idx + 5)..]; // Skip past ".log."
            return suffix
                .chars()
                .all(|c| c.is_ascii_digit() || c == '-' || c == '_');
        }

        false
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Import for tests
    use std::fs::File;
    use std::io::Write;
    use std::time::{Duration as StdDuration, SystemTime};

    #[test]
    fn test_is_log_file() {
        // Test various file paths
        assert!(is_log_file(Path::new("app.log")));
        assert!(is_log_file(Path::new("app.log.2023-01-01")));
        assert!(is_log_file(Path::new("/tmp/logs/app/test.log")));
        assert!(!is_log_file(Path::new("app.txt")));
        assert!(!is_log_file(Path::new("app.log.txt")));
        assert!(!is_log_file(Path::new("logfile")));
    }

    #[test]
    fn test_cleanup_old_files() {
        // Create a temporary directory for testing
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create some test files with different modification times
        let now = SystemTime::now();

        // Create a recent file that should NOT be deleted
        let recent_file = temp_path.join("recent.log");
        let mut file = File::create(&recent_file).unwrap();
        file.write_all(b"recent log file").unwrap();

        // Create an old file that SHOULD be deleted (10 days old)
        let old_file = temp_path.join("old.log");
        let mut file = File::create(&old_file).unwrap();
        file.write_all(b"old log file").unwrap();
        let old_time = now - StdDuration::from_secs(10 * 24 * 60 * 60);
        filetime::set_file_mtime(&old_file, filetime::FileTime::from_system_time(old_time))
            .unwrap();

        // Create a very old file that SHOULD be deleted (20 days old)
        let very_old_file = temp_path.join("very_old.log");
        let mut file = File::create(&very_old_file).unwrap();
        file.write_all(b"very old log file").unwrap();
        let very_old_time = now - StdDuration::from_secs(20 * 24 * 60 * 60);
        filetime::set_file_mtime(
            &very_old_file,
            filetime::FileTime::from_system_time(very_old_time),
        )
        .unwrap();

        // Create a non-log file that should NOT be deleted regardless of age
        let non_log_file = temp_path.join("data.txt");
        let mut file = File::create(&non_log_file).unwrap();
        file.write_all(b"not a log file").unwrap();
        filetime::set_file_mtime(
            &non_log_file,
            filetime::FileTime::from_system_time(very_old_time),
        )
        .unwrap();

        // Run cleanup with 7 days max age
        let result = cleanup_directory(temp_path, 7);

        // Check the results
        assert_eq!(result.files_removed, 2); // Both old and very_old should be removed
        assert!(result.bytes_removed > 0);
        assert!(result.failed_files.is_empty());

        // Verify the right files still exist
        assert!(recent_file.exists());
        assert!(!old_file.exists());
        assert!(!very_old_file.exists());
        assert!(non_log_file.exists());
    }

    #[test]
    fn test_cleanup_results_merge() {
        // Create two results
        let mut result1 = CleanupResult {
            files_removed: 5,
            bytes_removed: 1000,
            failed_files: vec![(PathBuf::from("/path/to/file1"), "error1".to_string())],
        };

        let result2 = CleanupResult {
            files_removed: 3,
            bytes_removed: 500,
            failed_files: vec![(PathBuf::from("/path/to/file2"), "error2".to_string())],
        };

        // Merge them
        result1.merge(result2);

        // Check the merged result
        assert_eq!(result1.files_removed, 8);
        assert_eq!(result1.bytes_removed, 1500);
        assert_eq!(result1.failed_files.len(), 2);
    }
}
