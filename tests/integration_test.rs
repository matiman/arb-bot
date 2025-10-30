#[cfg(test)]
mod tests {
    use std::process::Command;

    #[test]
    fn test_hello_world_output() {
        // Build the binary
        let build_output = Command::new("cargo")
            .args(["build", "--release"]) // remove needless borrow
            .output()
            .expect("Failed to build the project");

        assert!(
            build_output.status.success(),
            "Build failed: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );

        // Run the binary and capture output
        let output = Command::new("./target/release/arb-bot")
            .output()
            .expect("Failed to execute binary");

        // Verify the binary executed successfully
        assert!(
            output.status.success(),
            "Binary execution failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        // Convert output to string and verify it contains "Hello, world!"
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Hello, world!"),
            "Expected 'Hello, world!' in output, got: {}",
            stdout
        );
    }

    #[test]
    fn test_hello_world_basic() {
        // For development builds, test with cargo run
        let output = Command::new("cargo")
            .args(["run", "--quiet"]) // remove needless borrow
            .output()
            .expect("Failed to run the project");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Hello, world!"),
            "Expected 'Hello, world!' in output, got: {}",
            stdout
        );
    }
}
