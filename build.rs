//! Build script for kql-language-tools
//!
//! This script automatically builds the .NET native library if:
//! 1. The native library doesn't exist
//! 2. The .NET SDK is available
//!
//! If the .NET SDK isn't available, it provides helpful instructions.

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Set rerun triggers for .NET source files
    println!("cargo:rerun-if-changed=dotnet/src/");
    println!("cargo:rerun-if-changed=dotnet/KqlLanguageFfi.csproj");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Skip build during cargo publish verification - we can't write outside OUT_DIR
    // Cargo extracts the package to target/package/<name>-<version>/ for verification
    if manifest_dir
        .components()
        .any(|c| c.as_os_str() == "package")
    {
        println!("cargo:warning=Skipping native build during package verification");
        return;
    }
    let dotnet_dir = manifest_dir.join("dotnet");

    // Determine current platform RID
    let rid = current_rid();
    let lib_name = native_lib_name();

    // Check if native library already exists
    let native_dir = dotnet_dir.join("native").join(rid);
    let native_lib_path = native_dir.join(lib_name);

    if native_lib_path.exists() {
        println!(
            "cargo:warning=Native library found at {}",
            native_lib_path.display()
        );
        return;
    }

    // Check if KQL_LANGUAGE_TOOLS_PATH is set (user-provided library)
    if env::var("KQL_LANGUAGE_TOOLS_PATH").is_ok() {
        println!("cargo:warning=KQL_LANGUAGE_TOOLS_PATH is set, skipping native build");
        return;
    }

    // Native library doesn't exist - try to build it
    println!("cargo:warning=Native library not found, attempting to build...");

    // Check if dotnet SDK is available
    if !is_dotnet_available() {
        print_dotnet_instructions(rid, lib_name);
        return;
    }

    // Build using dotnet publish directly (cross-platform)
    println!("cargo:warning=Building native library for {rid}...");

    // Ensure native output directory exists
    if let Err(e) = std::fs::create_dir_all(&native_dir) {
        println!("cargo:warning=Failed to create output directory: {e}");
        print_manual_build_instructions(rid);
        return;
    }

    // Run dotnet publish
    let output = Command::new("dotnet")
        .args([
            "publish",
            "-c",
            "Release",
            "-r",
            rid,
            "-o",
            native_dir.to_str().unwrap_or("native"),
        ])
        .current_dir(&dotnet_dir)
        .output();

    match output {
        Ok(result) if result.status.success() => {
            // Copy the DNNE native export library from build artifacts
            let dnne_lib_path = dotnet_dir
                .join("obj")
                .join("Release")
                .join("net8.0")
                .join(rid)
                .join("dnne")
                .join("bin")
                .join(lib_name);

            if dnne_lib_path.exists() {
                if let Err(e) = std::fs::copy(&dnne_lib_path, &native_lib_path) {
                    println!("cargo:warning=Failed to copy DNNE library: {e}");
                    print_manual_build_instructions(rid);
                    return;
                }
            }

            // Verify the library was actually created
            if native_lib_path.exists() {
                println!("cargo:warning=Native library built successfully");
                println!(
                    "cargo:warning=Native library available at {}",
                    native_lib_path.display()
                );

                // Patch runtime config for major version rollforward
                let config_path = native_dir.join("KqlLanguageFfi.runtimeconfig.json");
                if config_path.exists() {
                    patch_runtime_config(&config_path);
                }
            } else {
                // Build claimed success but library doesn't exist
                println!("cargo:warning=Build completed but native library not found!");
                println!("cargo:warning=Expected: {}", native_lib_path.display());
                println!(
                    "cargo:warning=DNNE path checked: {}",
                    dnne_lib_path.display()
                );
                print_build_output(&result.stdout, &result.stderr);
                print_manual_build_instructions(rid);
            }
        }
        Ok(result) => {
            println!(
                "cargo:warning=Native library build failed with exit code: {:?}",
                result.status.code()
            );
            print_build_output(&result.stdout, &result.stderr);
            print_manual_build_instructions(rid);
        }
        Err(e) => {
            println!("cargo:warning=Failed to run dotnet publish: {e}");
            print_manual_build_instructions(rid);
        }
    }
}

/// Print build output for debugging
fn print_build_output(stdout: &[u8], stderr: &[u8]) {
    let stdout_str = String::from_utf8_lossy(stdout);
    let stderr_str = String::from_utf8_lossy(stderr);

    if !stdout_str.is_empty() {
        for line in stdout_str.lines().take(20) {
            println!("cargo:warning=[dotnet] {line}");
        }
    }
    if !stderr_str.is_empty() {
        for line in stderr_str.lines().take(10) {
            println!("cargo:warning=[dotnet-err] {line}");
        }
    }
}

/// Patch runtime config to allow major version rollforward
fn patch_runtime_config(config_path: &PathBuf) {
    if let Ok(content) = std::fs::read_to_string(config_path) {
        // Replace rollForward value with "Major" to allow running on newer .NET versions
        let patched = content.replace(
            r#""rollForward": "LatestMinor""#,
            r#""rollForward": "Major""#,
        );
        if patched != content {
            if let Err(e) = std::fs::write(config_path, patched) {
                println!("cargo:warning=Failed to patch runtime config: {e}");
            } else {
                println!("cargo:warning=Patched runtime config for major version rollforward");
            }
        }
    }
}

/// Get the runtime identifier for the current platform
fn current_rid() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return "osx-arm64";

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return "osx-x64";

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return "linux-x64";

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    return "linux-arm64";

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return "win-x64";

    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    return "win-arm64";
}

/// Get the native library filename for the current platform
fn native_lib_name() -> &'static str {
    #[cfg(target_os = "macos")]
    return "KqlLanguageFfiNE.dylib";

    #[cfg(target_os = "linux")]
    return "KqlLanguageFfiNE.so";

    #[cfg(target_os = "windows")]
    return "KqlLanguageFfiNE.dll";
}

/// Check if the dotnet SDK is available
fn is_dotnet_available() -> bool {
    Command::new("dotnet")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Print instructions for installing .NET SDK
fn print_dotnet_instructions(rid: &str, lib_name: &str) {
    println!("cargo:warning=");
    println!("cargo:warning======================================================");
    println!("cargo:warning=.NET SDK not found - cannot build native library");
    println!("cargo:warning======================================================");
    println!("cargo:warning=");
    println!("cargo:warning=The kql-language-tools crate requires a native library built from .NET.");
    println!("cargo:warning=");
    println!("cargo:warning=Options:");
    println!("cargo:warning=");
    println!("cargo:warning=1. Install .NET 8.0+ SDK and rebuild:");
    println!("cargo:warning=   - macOS: brew install dotnet");
    println!("cargo:warning=   - Linux: https://docs.microsoft.com/dotnet/core/install/linux");
    println!("cargo:warning=   - Windows: https://dotnet.microsoft.com/download");
    println!("cargo:warning=");
    println!("cargo:warning=2. Set KQL_LANGUAGE_TOOLS_PATH to a pre-built library:");
    println!("cargo:warning=   export KQL_LANGUAGE_TOOLS_PATH=/path/to/{lib_name}");
    println!("cargo:warning=");
    println!("cargo:warning=3. Download pre-built binaries from releases (if available)");
    println!("cargo:warning=");
    println!("cargo:warning=Target platform: {rid} ({lib_name})");
    println!("cargo:warning======================================================");
}

/// Print instructions for manual build
fn print_manual_build_instructions(rid: &str) {
    println!("cargo:warning=");
    println!("cargo:warning=To build manually, run:");
    println!("cargo:warning=  cd dotnet");
    println!("cargo:warning=  dotnet publish -c Release -r {rid}");
    println!("cargo:warning=");
    println!("cargo:warning=Or use the shell script (macOS/Linux):");
    println!("cargo:warning=  cd dotnet && ./build.sh {rid}");
}
