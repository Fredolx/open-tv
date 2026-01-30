use anyhow::{Context, Result};
use serde::Serialize;
use std::env::consts::OS;
use std::process::Command;
use which::which;

/// Represents the status of a single dependency
#[derive(Debug, Clone, Serialize)]
pub struct DependencyStatus {
    pub name: String,
    pub installed: bool,
    pub path: Option<String>,
    pub version: Option<String>,
}

/// Represents the overall dependency check result
#[derive(Debug, Clone, Serialize)]
pub struct DependencyCheckResult {
    pub all_satisfied: bool,
    pub dependencies: Vec<DependencyStatus>,
    pub platform: String,
    pub install_instructions: Option<String>,
}

const REQUIRED_DEPS: [&str; 3] = ["mpv", "ffmpeg", "yt-dlp"];

/// Check if a binary exists in PATH or bundled deps folder
fn check_binary(name: &str) -> DependencyStatus {
    // First check system PATH
    if let Ok(path) = which(name) {
        let version = get_version(name);
        return DependencyStatus {
            name: name.to_string(),
            installed: true,
            path: Some(path.to_string_lossy().to_string()),
            version,
        };
    }

    // On Windows, check bundled deps folder
    #[cfg(target_os = "windows")]
    {
        if let Ok(exe_path) = std::env::current_exe() {
            let mut deps_path = exe_path.clone();
            deps_path.pop();
            deps_path.push("deps");
            deps_path.push(format!("{}.exe", name));
            if deps_path.exists() {
                return DependencyStatus {
                    name: name.to_string(),
                    installed: true,
                    path: Some(deps_path.to_string_lossy().to_string()),
                    version: None,
                };
            }
        }
    }

    // On macOS, check common Homebrew/MacPorts paths
    #[cfg(target_os = "macos")]
    {
        let macos_paths = [
            "/opt/homebrew/bin",
            "/usr/local/bin",
            "/opt/local/bin",
        ];
        for base_path in macos_paths {
            let full_path = format!("{}/{}", base_path, name);
            if std::path::Path::new(&full_path).exists() {
                let version = get_version(&full_path);
                return DependencyStatus {
                    name: name.to_string(),
                    installed: true,
                    path: Some(full_path),
                    version,
                };
            }
        }
    }

    DependencyStatus {
        name: name.to_string(),
        installed: false,
        path: None,
        version: None,
    }
}

/// Try to get version string from a binary
fn get_version(binary: &str) -> Option<String> {
    let output = Command::new(binary)
        .arg("--version")
        .output()
        .ok()?;
    
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Return first line of version output
        stdout.lines().next().map(|s| s.to_string())
    } else {
        None
    }
}

/// Generate platform-specific installation instructions
fn get_install_instructions(missing: &[&str]) -> String {
    if missing.is_empty() {
        return String::new();
    }

    let deps_list = missing.join(" ");

    match OS {
        "macos" => format!(
            "Missing dependencies: {}\n\n\
            Install using Homebrew:\n\
            brew install {}\n\n\
            Or using MacPorts:\n\
            sudo port install {}",
            missing.join(", "),
            deps_list,
            deps_list
        ),
        "linux" => format!(
            "Missing dependencies: {}\n\n\
            Debian/Ubuntu:\n\
            sudo apt install {}\n\n\
            Fedora:\n\
            sudo dnf install {}\n\n\
            Arch Linux:\n\
            sudo pacman -S {}",
            missing.join(", "),
            deps_list,
            deps_list,
            deps_list
        ),
        "windows" => format!(
            "Missing dependencies: {}\n\n\
            These should be bundled with the installer.\n\
            Try reinstalling the application, or install manually:\n\n\
            Using Scoop:\n\
            scoop install {}\n\n\
            Using Chocolatey:\n\
            choco install {}",
            missing.join(", "),
            deps_list,
            deps_list
        ),
        _ => format!(
            "Missing dependencies: {}\n\n\
            Please install these manually for your platform.",
            missing.join(", ")
        ),
    }
}

/// Check all required dependencies and return the result
pub fn check_dependencies() -> DependencyCheckResult {
    let mut dependencies = Vec::new();
    let mut missing: Vec<&str> = Vec::new();

    for dep in REQUIRED_DEPS {
        let status = check_binary(dep);
        if !status.installed {
            missing.push(dep);
        }
        dependencies.push(status);
    }

    let all_satisfied = missing.is_empty();
    let install_instructions = if all_satisfied {
        None
    } else {
        Some(get_install_instructions(&missing))
    };

    DependencyCheckResult {
        all_satisfied,
        dependencies,
        platform: OS.to_string(),
        install_instructions,
    }
}

/// Check if this is the first run (no database exists yet)
pub fn is_first_run() -> bool {
    use directories::ProjectDirs;
    
    if let Some(proj_dirs) = ProjectDirs::from("dev", "fredol", "open-tv") {
        let db_path = proj_dirs.data_dir().join("beatstv.db");
        !db_path.exists()
    } else {
        true
    }
}
