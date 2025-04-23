// This file contains deliberate clippy errors for testing the pre-commit hook

#[allow(dead_code)]
fn function_with_clippy_warning() {
    // Unused variable should trigger a clippy warning
    let unused_variable = 5;
    
    // Redundant cloning should trigger a clippy warning
    let s = String::from("Hello");
    let _s2 = s.clone();
}