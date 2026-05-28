//! Runtime download and installation of the mihomo core binary.
//!
//! Fetches mihomo from [MetaCubeX/mihomo](https://github.com/MetaCubeX/mihomo)
//! GitHub releases, decompresses, and places the binary under the managed core
//! directory (`<data_dir>/core/`).

use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};

const VERSION_URL: &str =
    "https://github.com/MetaCubeX/mihomo/releases/latest/download/version.txt";

// ── Platform constants ──────────────────────────────────────────────────────

pub const fn platform_key() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "darwin-arm64"
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "darwin-amd64"
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "linux-amd64"
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        "linux-arm64"
    }
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        "windows-amd64"
    }
}

pub const fn archive_ext() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "zip"
    }
    #[cfg(not(target_os = "windows"))]
    {
        "gz"
    }
}

const fn exe_ext() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        ".exe"
    }
    #[cfg(not(target_os = "windows"))]
    {
        ""
    }
}

// ── Path helpers ────────────────────────────────────────────────────────────

/// Managed core binary: `<data_dir>/core/mihomo` (or `.exe` on Windows).
pub fn managed_core_path(data_dir: &Path) -> PathBuf {
    data_dir.join("core").join(format!("mihomo{}", exe_ext()))
}

/// Version tracking file: `<data_dir>/core/version.txt`.
pub fn version_file_path(data_dir: &Path) -> PathBuf {
    data_dir.join("core").join("version.txt")
}

/// Read the locally installed version, if version.txt exists and is non-empty.
pub fn installed_version(data_dir: &Path) -> Option<String> {
    let path = version_file_path(data_dir);
    std::fs::read_to_string(&path)
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

/// A minimal progress callback: a human-readable status string.
pub type ProgressFn = dyn Fn(&str) + Send + Sync;

// ── Network ─────────────────────────────────────────────────────────────────

/// Fetch the latest stable version string from GitHub.
///
/// Returns e.g. `"1.19.25"`.
pub async fn fetch_latest_version() -> Result<String> {
    let resp = reqwest::get(VERSION_URL)
        .await
        .context("failed to contact GitHub for version check")?;
    if !resp.status().is_success() {
        bail!("version check returned HTTP {}", resp.status());
    }
    let text = resp
        .text()
        .await
        .context("failed to read version response")?;
    let mut version = text.trim().to_owned();
    if version.is_empty() {
        bail!("empty version response");
    }
    // version.txt returns e.g. "v1.19.25" — strip the 'v' prefix so downstream
    // URL construction (v{version}) doesn't double up.
    if let Some(stripped) = version.strip_prefix('v') {
        version = stripped.to_owned();
    }
    log::info!("latest mihomo version: {version}");
    Ok(version)
}

// ── Download + install ──────────────────────────────────────────────────────

/// Download and install mihomo for the given version.
///
/// Returns the path to the installed binary. Calls `on_progress` with
/// human-readable status messages during the operation.
pub async fn install_version(
    data_dir: &Path,
    version: &str,
    on_progress: &ProgressFn,
) -> Result<PathBuf> {
    let core_dir = data_dir.join("core");
    std::fs::create_dir_all(&core_dir).context("failed to create core directory")?;

    let binary_id = format!("mihomo-{}-v{version}", platform_key());
    let archive_name = format!("{binary_id}.{}", archive_ext());
    let download_url =
        format!("https://github.com/MetaCubeX/mihomo/releases/download/v{version}/{archive_name}");
    let temp_archive = core_dir.join(&archive_name);
    let target = managed_core_path(data_dir);

    on_progress(&format!("downloading mihomo v{version}..."));
    download_file(&download_url, &temp_archive)
        .await
        .with_context(|| format!("failed to download from {download_url}"))?;

    on_progress("extracting...");
    let ext = archive_ext();
    if ext == "zip" {
        decompress_zip(&temp_archive, &target)?;
    } else {
        decompress_gz(&temp_archive, &target)?;
    }

    let _ = std::fs::remove_file(&temp_archive);

    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o755))
            .context("failed to set executable permission")?;
    }

    std::fs::write(version_file_path(data_dir), version).context("failed to write version file")?;

    log::info!("mihomo v{version} installed to {}", target.display());
    Ok(target)
}

// ── Private helpers ─────────────────────────────────────────────────────────

async fn download_file(url: &str, dest: &Path) -> Result<()> {
    let resp = reqwest::get(url).await?;
    if !resp.status().is_success() {
        bail!("download returned HTTP {} for {url}", resp.status());
    }
    let bytes = resp.bytes().await.context("failed to read download body")?;
    std::fs::write(dest, &bytes).context("failed to write downloaded file")?;
    Ok(())
}

fn decompress_gz(source: &Path, target: &Path) -> Result<()> {
    let file = std::fs::File::open(source).context("failed to open .gz archive")?;
    let mut decoder = flate2::read::GzDecoder::new(file);
    let mut buf = Vec::new();
    Read::read_to_end(&mut decoder, &mut buf).context("failed to decompress .gz archive")?;
    std::fs::write(target, &buf).context("failed to write decompressed binary")?;
    Ok(())
}

fn decompress_zip(source: &Path, target: &Path) -> Result<()> {
    let file = std::fs::File::open(source).context("failed to open .zip archive")?;
    let mut archive = zip::ZipArchive::new(file).context("failed to open zip archive")?;
    if archive.is_empty() {
        bail!("zip archive is empty");
    }
    let mut entry = archive.by_index(0).context("failed to read zip entry")?;
    let mut buf = Vec::new();
    Read::read_to_end(&mut entry, &mut buf).context("failed to read zip entry bytes")?;
    std::fs::write(target, &buf).context("failed to write decompressed binary")?;
    Ok(())
}
