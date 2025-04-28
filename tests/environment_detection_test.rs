use std::env;
use switchboard::logger::{detect_environment, LogEnvironment};

#[test]
fn test_development_environment_with_debug_assertions() {
    // This test verifies that Development is detected in debug mode
    // We can't really control debug_assertions at runtime, but we can
    // verify that the function returns a valid environment

    // The test should detect as Development when running in debug mode
    #[cfg(debug_assertions)]
    {
        let env = detect_environment();
        assert_eq!(
            env,
            LogEnvironment::Development,
            "Should detect as Development when debug_assertions are enabled"
        );
    }
}

#[test]
fn test_development_environment_with_env_var() {
    // Set the development environment variable
    env::set_var("SWITCHBOARD_DEV", "1");

    // Should detect as Development when the env var is set
    let env = detect_environment();
    assert_eq!(
        env,
        LogEnvironment::Development,
        "Should detect as Development when SWITCHBOARD_DEV is set"
    );

    // Clean up
    env::remove_var("SWITCHBOARD_DEV");
}

#[test]
fn test_system_service_environment_simulation() {
    // We can't easily simulate a real system service in tests,
    // but we can test the environment variable logic for platforms

    #[cfg(target_os = "linux")]
    {
        // First save any existing env vars to restore later
        let journal_stream_orig = env::var("JOURNAL_STREAM").ok();

        // Set system service environment variable for Linux
        env::set_var("JOURNAL_STREAM", "1");

        let env = detect_environment();
        if !cfg!(debug_assertions) {
            // Only check in release mode since debug_assertions takes precedence
            assert_eq!(
                env,
                LogEnvironment::SystemService,
                "Should detect as SystemService when JOURNAL_STREAM is set"
            );
        }

        // Restore original env vars
        match journal_stream_orig {
            Some(val) => env::set_var("JOURNAL_STREAM", val),
            None => env::remove_var("JOURNAL_STREAM"),
        }
    }

    #[cfg(target_os = "macos")]
    {
        // First save any existing env vars to restore later
        let xpc_service_orig = env::var("XPC_SERVICE_NAME").ok();

        // Set system service environment variable for macOS
        env::set_var("XPC_SERVICE_NAME", "com.example.service");

        let env = detect_environment();
        if !cfg!(debug_assertions) {
            // Only check in release mode since debug_assertions takes precedence
            assert_eq!(
                env,
                LogEnvironment::SystemService,
                "Should detect as SystemService when XPC_SERVICE_NAME is set"
            );
        }

        // Restore original env vars
        match xpc_service_orig {
            Some(val) => env::set_var("XPC_SERVICE_NAME", val),
            None => env::remove_var("XPC_SERVICE_NAME"),
        }
    }

    #[cfg(target_os = "windows")]
    {
        // First save any existing env vars to restore later
        let windir_orig = env::var("WINDIR").ok();
        let userprofile_orig = env::var("USERPROFILE").ok();

        // Set system service environment variables for Windows
        env::set_var("WINDIR", "C:\\Windows");
        env::remove_var("USERPROFILE");

        let env = detect_environment();
        if !cfg!(debug_assertions) {
            // Only check in release mode since debug_assertions takes precedence
            assert_eq!(
                env,
                LogEnvironment::SystemService,
                "Should detect as SystemService with Windows service environment"
            );
        }

        // Restore original env vars
        match windir_orig {
            Some(val) => env::set_var("WINDIR", val),
            None => env::remove_var("WINDIR"),
        }
        match userprofile_orig {
            Some(val) => env::set_var("USERPROFILE", val),
            None => {}
        }
    }
}

#[test]
fn test_user_installation_environment_simulation() {
    // First save any existing env vars to restore later
    let switchboard_dev_orig = env::var("SWITCHBOARD_DEV").ok();
    let home_orig = env::var("HOME").ok();

    // Ensure Development mode is not triggered
    env::remove_var("SWITCHBOARD_DEV");

    // Mock a user installation situation where executable is in HOME
    if let Ok(_home) = env::var("HOME").or_else(|_| env::var("USERPROFILE")) {
        // Only proceed if we have a home directory

        // Save the real exe path
        let real_exe = std::env::current_exe().ok();

        // We can't change the actual exe path, but we can set HOME to include
        // the current exe path, simulating that the exe is in HOME
        if let Some(exe_path) = real_exe {
            if let Some(exe_parent) = exe_path.parent() {
                // Set HOME to the parent of the executable
                env::set_var("HOME", exe_parent.to_string_lossy().to_string());

                let env = detect_environment();
                if !cfg!(debug_assertions) {
                    // Only check in release mode since debug_assertions takes precedence
                    assert_eq!(
                        env,
                        LogEnvironment::UserInstallation,
                        "Should detect as UserInstallation when executable is in HOME"
                    );
                }
            }
        }

        // Restore HOME
        match home_orig {
            Some(val) => env::set_var("HOME", val),
            None => env::remove_var("HOME"),
        }
    }

    // Restore other env vars
    if let Some(val) = switchboard_dev_orig {
        env::set_var("SWITCHBOARD_DEV", val)
    }
}

#[test]
fn test_fallback_to_development() {
    // We'll mock a scenario where no specific environment is detected
    // by clearing all the environment variables that would trigger
    // specific environment detection

    // Save the original environment variables
    let vars_to_save = [
        "SWITCHBOARD_DEV",
        "HOME",
        "USERPROFILE",
        "JOURNAL_STREAM",
        "INVOCATION_ID",
        "XPC_SERVICE_NAME",
        "WINDIR",
        "SESSIONNAME",
    ];

    let mut saved_vars = std::collections::HashMap::new();
    for var in vars_to_save {
        saved_vars.insert(var, env::var(var).ok());
    }

    // Clear all environment variables that would trigger specific detection
    for var in vars_to_save {
        env::remove_var(var);
    }

    // In the absence of any specific indicators, it should fall back to Development
    let env = detect_environment();
    if !cfg!(debug_assertions) {
        // Only check in release mode since debug_assertions takes precedence
        assert_eq!(
            env,
            LogEnvironment::Development,
            "Should fall back to Development when no specific environment is detected"
        );
    }

    // Restore the original environment variables
    for (var, value_opt) in saved_vars {
        if let Some(value) = value_opt {
            env::set_var(var, value);
        }
    }
}
