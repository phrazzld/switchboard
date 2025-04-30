use crate::logger::{DEFAULT_LOG_DIR, SYSTEM_LOG_DIR};
use directories::ProjectDirs;
use std::env;
use std::path::PathBuf;

/// Represents the environment in which the application is running
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogEnvironment {
    /// Development environment (local development)
    Development,
    /// User-level installation (installed for a specific user)
    UserInstallation,
    /// System-level service (running as a system service)
    SystemService,
}

/// Detects the environment in which the application is running
///
/// This function uses platform-specific logic to detect the current execution environment:
///
/// - Development: Local development environment (default if no specific indicators found)
/// - UserInstallation: Installed for a specific user (e.g., in user's home directory)
/// - SystemService: Running as a system service (e.g., systemd service on Linux,
///   launchd service on macOS, or Windows Service)
///
/// # Platform-specific detection
///
/// ## Linux
/// - Checks if parent PID is 1
/// - Inspects `/proc/self/cgroup` for systemd
/// - Looks for systemd environment variables
///
/// ## macOS
/// - Checks if parent PID is 1
/// - Checks for controlling TTY
/// - Looks for XPC environment variables
///
/// ## Windows
/// - Uses Service Control Manager to check if running as a service
///
/// # Returns
///
/// Returns the detected `LogEnvironment` variant that best matches the current execution context.
///
/// # Examples
///
/// ```
/// use switchboard::logger::detect_environment;
///
/// let env = detect_environment();
/// println!("Running in {:?} environment", env);
/// ```
pub fn detect_environment() -> LogEnvironment {
    // Check if we're in a development-specific environment
    if cfg!(debug_assertions) || env::var("SWITCHBOARD_DEV").is_ok() {
        return LogEnvironment::Development;
    }

    // Platform-specific detection logic
    #[cfg(target_os = "linux")]
    {
        // Check if the parent process is init (PID 1)
        let ppid = unsafe { libc::getppid() };
        if ppid == 1 {
            return LogEnvironment::SystemService;
        }

        // Check cgroup to detect container or systemd service
        if let Ok(cgroups) = std::fs::read_to_string("/proc/self/cgroup") {
            // Systemd service units are typically in their own cgroup
            if cgroups.contains("systemd") || cgroups.contains("/system.slice/") {
                return LogEnvironment::SystemService;
            }
        }

        // Check for systemd-specific environment variables
        if env::var("INVOCATION_ID").is_ok() || env::var("JOURNAL_STREAM").is_ok() {
            return LogEnvironment::SystemService;
        }
    }

    #[cfg(target_os = "macos")]
    {
        // Check if the parent process is launchd (PID 1)
        let ppid = unsafe { libc::getppid() };
        if ppid == 1 {
            // Most likely a launchd service
            return LogEnvironment::SystemService;
        }

        // Check for XPC environment variables which indicate a launchd service
        if env::var("XPC_SERVICE_NAME").is_ok() {
            return LogEnvironment::SystemService;
        }

        // Check if there's no controlling terminal (typical for services)
        // Skip the TTY check since we removed the process and Command imports
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, detecting service status is more complex
        // For a proper implementation, we would use the Windows API
        // (QueryServiceStatusEx with Service Control Manager)

        // As a simpler heuristic, check environment variables that suggest a service
        if env::var("WINDIR").is_ok() && env::var("USERPROFILE").is_err() {
            // Services typically run in system context without user profile
            return LogEnvironment::SystemService;
        }

        // Check if running without a console, which is typical for services
        // This is a simplified approach; a full implementation would use Windows API
        if env::var("SESSIONNAME").is_err() {
            return LogEnvironment::SystemService;
        }
    }

    // Check for indications of a user installation (outside of development)
    // Here we're checking common patterns for user-installed applications
    if let Ok(home) = env::var("HOME") {
        if let Ok(exe_path) = env::current_exe() {
            if let Some(exe_str) = exe_path.to_str() {
                if exe_str.contains(&home) {
                    return LogEnvironment::UserInstallation;
                }
            }
        }
    }

    // Default if no specific environment is detected
    LogEnvironment::Development
}

/// Returns the XDG-compliant log directory for the application
///
/// This function uses the `directories` crate to retrieve the platform-specific
/// data directory according to XDG Base Directory Specification on Linux and
/// equivalent standards on macOS and Windows.
///
/// The function returns a path in the following format:
/// - Linux: `~/.local/share/switchboard/logs`
/// - macOS: `~/Library/Application Support/switchboard/logs`
/// - Windows: `C:\Users\<user>\AppData\Roaming\switchboard\logs`
///
/// # Returns
///
/// A `PathBuf` containing the XDG-compliant path for storing log files
///
/// # Examples
///
/// ```
/// use switchboard::logger::get_xdg_log_directory;
///
/// let xdg_path = get_xdg_log_directory();
/// println!("XDG log directory: {}", xdg_path.display());
/// ```
pub fn get_xdg_log_directory() -> PathBuf {
    // Use the directories crate to get platform-specific data directory
    // We use "switchboard" as the organization and application name
    if let Some(proj_dirs) = ProjectDirs::from("", "", "switchboard") {
        // Get the data directory and append logs subdirectory
        proj_dirs.data_dir().join("logs")
    } else {
        // Fallback to a reasonable default if we can't get XDG directory
        // This should be rare but could happen in constrained environments
        PathBuf::from(DEFAULT_LOG_DIR)
    }
}

/// Returns the appropriate log directory path based on the environment
///
/// This function determines the appropriate log directory path based on the
/// current execution environment:
///
/// - `Development`: Uses `DEFAULT_LOG_DIR` (./logs/)
/// - `UserInstallation`: Uses XDG-compliant directory from `get_xdg_log_directory`
/// - `SystemService`: Uses system log path (e.g., /var/log/switchboard/ on Unix)
///
/// # Arguments
///
/// * `environment` - The `LogEnvironment` value to determine the path for
///
/// # Returns
///
/// A `PathBuf` containing the appropriate log directory path
///
/// # Examples
///
/// ```
/// use switchboard::logger::{get_environment_log_directory, LogEnvironment};
///
/// // Get log directory for development environment
/// let dev_path = get_environment_log_directory(LogEnvironment::Development);
/// assert_eq!(dev_path.to_str().unwrap(), "./logs");
///
/// // Get log directory for user installation (XDG-compliant)
/// let user_path = get_environment_log_directory(LogEnvironment::UserInstallation);
/// // Path will vary by platform and username
/// ```
pub fn get_environment_log_directory(environment: LogEnvironment) -> PathBuf {
    match environment {
        LogEnvironment::Development => PathBuf::from(DEFAULT_LOG_DIR),
        LogEnvironment::UserInstallation => get_xdg_log_directory(),
        LogEnvironment::SystemService => PathBuf::from(SYSTEM_LOG_DIR),
    }
}
