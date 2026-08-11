use std::env;
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, PartialEq)]
pub struct UpdateInfo {
    pub version: String,
    pub release_notes: String,
    pub download_url: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UpdateStatus {
    Idle,
    Checking,
    Available(UpdateInfo),
    UpToDate,
    Throttled,
    Downloading,
    Error(String),
}

pub fn parse_semver(v: &str) -> (u32, u32, u32) {
    let clean = v.trim().strip_prefix('v').unwrap_or(v.trim());
    let mut parts = clean.split('.');
    let major = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    (major, minor, patch)
}

pub fn is_newer_version(current: &str, remote: &str) -> bool {
    let (c_maj, c_min, c_pat) = parse_semver(current);
    let (r_maj, r_min, r_pat) = parse_semver(remote);

    if r_maj != c_maj {
        return r_maj > c_maj;
    }
    if r_min != c_min {
        return r_min > c_min;
    }
    r_pat > c_pat
}

pub fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn check_latest_release(
    is_manual: bool,
    current_version: &str,
    last_check_timestamp: u64,
) -> (UpdateStatus, u64) {
    let now = get_current_timestamp();

    // 12 hours = 43,200 seconds
    if !is_manual && last_check_timestamp > 0 && now.saturating_sub(last_check_timestamp) < 43200 {
        return (UpdateStatus::Throttled, last_check_timestamp);
    }

    let ps_script = format!(
        "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; \
         $headers = @{{ 'User-Agent' = 'Audio-Tetris-App/{}' }}; \
         $resp = Invoke-RestMethod -Uri 'https://api.github.com/repos/thegregariousdragon/audio-tetris/releases/latest' -Headers $headers -TimeoutSec 10; \
         $asset = $resp.assets | Where-Object {{ $_.name -like '*audio-tetris*.zip' -or $_.name -like '*.zip' }} | Select-Object -First 1; \
         $out = @{{ tag_name = $resp.tag_name; body = $resp.body; download_url = $asset.browser_download_url }}; \
         $out | ConvertTo-Json -Compress",
        current_version
    );

    let output = match Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_script])
        .output()
    {
        Ok(out) => out,
        Err(e) => {
            return (
                UpdateStatus::Error(format!("Failed to run update check: {}", e)),
                now,
            );
        }
    };

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let msg = if err_msg.is_empty() {
            "Could not connect to update server.".to_string()
        } else {
            err_msg
        };
        return (UpdateStatus::Error(msg), now);
    }

    let json_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if json_str.is_empty() {
        return (
            UpdateStatus::Error("No release data returned.".to_string()),
            now,
        );
    }

    let parsed: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(val) => val,
        Err(_) => {
            return (
                UpdateStatus::Error("Failed to parse release info.".to_string()),
                now,
            );
        }
    };

    let tag_name = parsed["tag_name"].as_str().unwrap_or("").to_string();
    let body = parsed["body"].as_str().unwrap_or("").to_string();
    let download_url = parsed["download_url"].as_str().unwrap_or("").to_string();

    if tag_name.is_empty() {
        return (
            UpdateStatus::Error("No version tag found in release.".to_string()),
            now,
        );
    }

    if is_newer_version(current_version, &tag_name) {
        let info = UpdateInfo {
            version: tag_name,
            release_notes: if body.trim().is_empty() {
                "No release notes provided.".to_string()
            } else {
                body
            },
            download_url,
        };
        (UpdateStatus::Available(info), now)
    } else {
        (UpdateStatus::UpToDate, now)
    }
}

pub fn perform_in_place_update(download_url: &str) -> Result<(), String> {
    if download_url.trim().is_empty() {
        return Err("No download URL provided for update.".to_string());
    }

    let temp_dir = env::temp_dir();
    let zip_path = temp_dir.join("audio_tetris_update.zip");
    let extract_dir = temp_dir.join("audio_tetris_update_staged");

    // Clean up old extraction directory if exists
    if extract_dir.exists() {
        let _ = fs::remove_dir_all(&extract_dir);
    }
    let _ = fs::create_dir_all(&extract_dir);

    // Download zip using PowerShell
    let ps_download = format!(
        "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; \
         Invoke-WebRequest -Uri '{}' -OutFile '{}' -UserAgent 'Audio-Tetris-App'",
        download_url,
        zip_path.to_string_lossy().replace('\\', "/")
    );

    let dl_output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_download])
        .output()
        .map_err(|e| format!("Failed to download update: {}", e))?;

    if !dl_output.status.success() {
        return Err(format!(
            "Download failed: {}",
            String::from_utf8_lossy(&dl_output.stderr)
        ));
    }

    // Extract zip using PowerShell
    let ps_extract = format!(
        "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
        zip_path.to_string_lossy().replace('\\', "/"),
        extract_dir.to_string_lossy().replace('\\', "/")
    );

    let ext_output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_extract])
        .output()
        .map_err(|e| format!("Failed to extract update package: {}", e))?;

    if !ext_output.status.success() {
        return Err(format!(
            "Extraction failed: {}",
            String::from_utf8_lossy(&ext_output.stderr)
        ));
    }

    // Locate staged folder ("Audio Tetris" nested folder or extract_dir root)
    let nested_dir = extract_dir.join("Audio Tetris");
    let source_dir = if nested_dir.exists() {
        nested_dir
    } else {
        extract_dir.clone()
    };

    let exe_in_source = source_dir.join("audio-tetris.exe");
    if !exe_in_source.exists() {
        return Err("Update package does not contain audio-tetris.exe".to_string());
    }

    // Get current executable directory
    let current_exe = env::current_exe().map_err(|e| format!("Failed to locate app dir: {}", e))?;
    let app_dir = current_exe
        .parent()
        .ok_or_else(|| "Failed to get app parent directory".to_string())?;

    // Create batch helper script for in-place copy & restart
    let batch_path = temp_dir.join("update_audio_tetris_helper.bat");
    let batch_content = format!(
        "@echo off\r\n\
         timeout /t 2 /nobreak > NUL\r\n\
         :retry\r\n\
         tasklist | find /i \"audio-tetris.exe\" > NUL\r\n\
         if %errorlevel% equ 0 (\r\n\
             timeout /t 1 /nobreak > NUL\r\n\
             goto retry\r\n\
         )\r\n\
         xcopy /s /e /y \"{}\\*\" \"{}\\\"\r\n\
         del /q \"{}\"\r\n\
         start \"\" \"{}\\audio-tetris.exe\"\r\n\
         del \"%~f0\"\r\n",
        source_dir.to_string_lossy(),
        app_dir.to_string_lossy(),
        zip_path.to_string_lossy(),
        app_dir.to_string_lossy()
    );

    fs::write(&batch_path, batch_content)
        .map_err(|e| format!("Failed to write updater script: {}", e))?;

    // Launch helper script detached
    let _ = Command::new("cmd")
        .args(["/C", "start", "", "/min", &batch_path.to_string_lossy()])
        .spawn();

    // Terminate current game process safely
    std::process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_semver() {
        assert_eq!(parse_semver("1.0.2"), (1, 0, 2));
        assert_eq!(parse_semver("v1.2.3"), (1, 2, 3));
        assert_eq!(parse_semver("v2.0.0-beta"), (2, 0, 0));
    }

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("1.0.2", "1.0.3"));
        assert!(is_newer_version("1.0.2", "1.1.0"));
        assert!(is_newer_version("1.0.2", "2.0.0"));
        assert!(!is_newer_version("1.0.2", "1.0.2"));
        assert!(!is_newer_version("1.0.3", "1.0.2"));
    }
}
