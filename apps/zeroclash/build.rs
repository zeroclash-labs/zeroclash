//! Build script: download the mihomo core binary into the output directory
//! so it is bundled alongside the zeroclash executable.
//!
//! Only runs for `--release` builds to keep debug builds fast.

use std::io::Read;
use std::path::{Path, PathBuf};

fn main() {
    let profile = std::env::var("PROFILE").unwrap_or_default();
    if profile != "release" {
        println!("cargo:warning=skipping mihomo core download (not a release build)");
        return;
    }

    println!("cargo:rerun-if-env-changed=PROFILE");

    let target = std::env::var("TARGET").unwrap_or_default();
    let (platform_key, archive_ext, exe_ext) = match_target(&target);

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let profile_dir = Path::new(&out_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("unexpected OUT_DIR structure");

    let dest = profile_dir.join(format!("mihomo{exe_ext}"));
    if dest.exists() {
        println!(
            "cargo:warning=mihomo core already exists at {}",
            dest.display()
        );
        return;
    }

    println!("cargo:warning=downloading mihomo core for {platform_key}...");

    let version = match fetch_version() {
        Ok(v) => v,
        Err(e) => {
            println!("cargo:warning=failed to fetch mihomo version: {e}");
            return;
        }
    };

    let binary_id = format!("mihomo-{platform_key}-v{version}");
    let archive_name = format!("{binary_id}.{archive_ext}");
    let url =
        format!("https://github.com/MetaCubeX/mihomo/releases/download/v{version}/{archive_name}");

    println!("cargo:warning=  fetching {url}");

    let temp = download_to_temp(&url, &out_dir, &archive_name);
    let temp = match temp {
        Ok(p) => p,
        Err(e) => {
            println!("cargo:warning=failed to download: {e}");
            return;
        }
    };

    match decompress(&temp, &dest, archive_ext) {
        Ok(()) => {
            let _ = std::fs::remove_file(&temp);
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755));
            }
            println!("cargo:warning=mihomo core installed to {}", dest.display());
        }
        Err(e) => {
            println!("cargo:warning=failed to decompress: {e}");
            let _ = std::fs::remove_file(&temp);
        }
    }
}

// ── Platform detection ─────────────────────────────────────────────────────

fn match_target(target: &str) -> (&'static str, &'static str, &'static str) {
    if target.contains("apple-darwin") {
        if target.starts_with("aarch64") {
            ("darwin-arm64", "gz", "")
        } else {
            ("darwin-amd64", "gz", "")
        }
    } else if target.contains("linux") {
        if target.starts_with("aarch64") {
            ("linux-arm64", "gz", "")
        } else {
            ("linux-amd64", "gz", "")
        }
    } else if target.contains("windows") {
        ("windows-amd64", "zip", ".exe")
    } else {
        // Best-effort fallback
        ("linux-amd64", "gz", "")
    }
}

// ── Network ─────────────────────────────────────────────────────────────────

fn fetch_version() -> Result<String, Box<dyn std::error::Error>> {
    let mut resp =
        ureq::get("https://github.com/MetaCubeX/mihomo/releases/latest/download/version.txt")
            .call()?;
    let text = resp.body_mut().read_to_string()?;
    let version = text.trim();
    // Strip 'v' prefix if present
    Ok(version.strip_prefix('v').unwrap_or(version).to_owned())
}

fn download_to_temp(
    url: &str,
    out_dir: &str,
    filename: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dest = Path::new(out_dir).join(filename);
    let resp = ureq::get(url).call()?;
    let (_parts, body) = resp.into_parts();
    let mut reader = body.into_reader();
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    std::fs::write(&dest, &buf)?;
    Ok(dest)
}

// ── Decompress ──────────────────────────────────────────────────────────────

fn decompress(src: &Path, dst: &Path, ext: &str) -> Result<(), Box<dyn std::error::Error>> {
    if ext == "zip" {
        let file = std::fs::File::open(src)?;
        let mut archive = zip::ZipArchive::new(file)?;
        let mut entry = archive.by_index(0)?;
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        std::fs::write(dst, &buf)?;
    } else {
        let file = std::fs::File::open(src)?;
        let mut decoder = flate2::read::GzDecoder::new(file);
        let mut buf = Vec::new();
        decoder.read_to_end(&mut buf)?;
        std::fs::write(dst, &buf)?;
    }
    Ok(())
}
