use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const DEFAULT_EMBED_URL: &str =
    "https://www.python.org/ftp/python/3.12.7/python-3.12.7-embed-amd64.zip";
const DEFAULT_GET_PIP_URL: &str = "https://bootstrap.pypa.io/get-pip.py";

fn main() {
    if !cfg!(target_os = "windows") {
        tauri_build::build();
        return;
    }


    if let Err(err) = ensure_resources_dir() {
        panic!("Failed to prepare resources dir: {err}");
    }

    if !should_bundle_embedded_python() {
        if let Ok(config) = fs::read_to_string("tauri.conf.dev.json") {
            env::set_var("TAURI_CONFIG", config);
        }
        tauri_build::build();
        return;
    }

    if let Err(err) = prepare_embedded_python() {
        panic!("Failed to prepare embedded Python: {err}");
    }

    tauri_build::build();
}

fn should_bundle_embedded_python() -> bool {
    if env::var("WEREPLY_BUNDLE_PYTHON")
        .ok()
        .as_deref()
        == Some("1")
    {
        return true;
    }
    env::var("PROFILE").map(|profile| profile == "release").unwrap_or(false)
}

fn prepare_embedded_python() -> io::Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let resources_root = manifest_dir.join("resources");
    let resources_dir = resources_root.join("python");
    let marker = resources_dir.join(".wereply_python_ready");

    if marker.exists() && resources_dir.join("python.exe").exists() {
        return Ok(());
    }

    if resources_dir.exists() {
        fs::remove_dir_all(&resources_dir)?;
    }
    fs::create_dir_all(&resources_dir)?;

    let cache_dir = manifest_dir.join("target").join("wereply-python-cache");
    fs::create_dir_all(&cache_dir)?;

    let embed_url = env::var("WEREPLY_PYTHON_EMBED_URL").unwrap_or_else(|_| DEFAULT_EMBED_URL.to_string());
    let get_pip_url = env::var("WEREPLY_GET_PIP_URL").unwrap_or_else(|_| DEFAULT_GET_PIP_URL.to_string());

    let embed_zip = cache_dir.join("python-embed.zip");
    download_if_missing(&embed_url, &embed_zip)?;
    unzip_archive(&embed_zip, &resources_dir)?;

    ensure_pth_allows_site_packages(&resources_dir)?;

    let get_pip = cache_dir.join("get-pip.py");
    download_if_missing(&get_pip_url, &get_pip)?;

    let python = resources_dir.join("python.exe");
    run_command(
        &python,
        &["get-pip.py", "--no-warn-script-location"],
        &resources_dir,
    )?;

    let requirements = manifest_dir
        .parent()
        .unwrap_or(&manifest_dir)
        .join("platform_agents")
        .join("windows")
        .join("requirements.txt");
    let site_packages = resources_dir.join("Lib").join("site-packages");
    fs::create_dir_all(&site_packages)?;

    run_command(
        &python,
        &[
            "-m",
            "pip",
            "install",
            "--disable-pip-version-check",
            "--no-input",
            "-r",
            requirements.to_string_lossy().as_ref(),
            "-t",
            site_packages.to_string_lossy().as_ref(),
        ],
        &resources_dir,
    )?;

    fs::write(marker, "ready")?;
    Ok(())
}

fn ensure_resources_dir() -> io::Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let resources_root = manifest_dir.join("resources");
    if !resources_root.exists() {
        fs::create_dir_all(&resources_root)?;
        fs::write(resources_root.join("placeholder.txt"), "placeholder")?;
    }
    if let Ok(current_dir) = env::current_dir() {
        let current_resources = current_dir.join("resources");
        if !current_resources.exists() {
            fs::create_dir_all(&current_resources)?;
            fs::write(current_resources.join("placeholder.txt"), "placeholder")?;
        }
    }
    Ok(())
}

fn download_if_missing(url: &str, dest: &Path) -> io::Result<()> {
    if dest.exists() {
        return Ok(());
    }
    let mut response =
        reqwest::blocking::get(url).map_err(|err| io::Error::other(format!("download failed: {err}")))?;
    let mut file = fs::File::create(dest)?;
    let mut buf = Vec::new();
    response
        .read_to_end(&mut buf)
        .map_err(|err| io::Error::other(format!("read failed: {err}")))?;
    file.write_all(&buf)?;
    Ok(())
}

fn unzip_archive(zip_path: &Path, dest: &Path) -> io::Result<()> {
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|err| io::Error::other(format!("zip open failed: {err}")))?;
    for i in 0..archive.len() {
        let mut entry =
            archive.by_index(i).map_err(|err| io::Error::other(format!("zip read failed: {err}")))?;
        let entry_path = entry
            .enclosed_name()
            .ok_or_else(|| io::Error::other("zip entry invalid"))?
            .to_owned();
        let out_path = dest.join(entry_path);
        if entry.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = fs::File::create(&out_path)?;
            io::copy(&mut entry, &mut outfile)?;
        }
    }
    Ok(())
}

fn ensure_pth_allows_site_packages(resources_dir: &Path) -> io::Result<()> {
    let pth = find_pth_file(resources_dir)?;
    let mut contents = String::new();
    fs::File::open(&pth)?.read_to_string(&mut contents)?;

    let mut lines: Vec<String> = contents.lines().map(|line| line.to_string()).collect();
    let mut has_site = lines.iter().any(|line| line.trim() == "Lib\\site-packages");
    let mut has_import_site = false;

    for line in &mut lines {
        if line.trim() == "#import site" {
            *line = "import site".to_string();
            has_import_site = true;
        } else if line.trim() == "import site" {
            has_import_site = true;
        }
    }

    if !has_site {
        lines.push("Lib\\site-packages".to_string());
        has_site = true;
    }
    if !has_import_site {
        lines.push("import site".to_string());
    }

    if has_site {
        let updated = lines.join("\n") + "\n";
        fs::write(&pth, updated)?;
    }
    Ok(())
}

fn find_pth_file(resources_dir: &Path) -> io::Result<PathBuf> {
    for entry in fs::read_dir(resources_dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with("python") && name.ends_with("._pth") {
                return Ok(path);
            }
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "python ._pth file not found",
    ))
}

fn run_command(python: &Path, args: &[&str], workdir: &Path) -> io::Result<()> {
    let output = Command::new(python)
        .args(args)
        .current_dir(workdir)
        .output()?;

    if output.status.success() {
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(io::Error::other(format!(
        "command failed stdout: {stdout} stderr: {stderr}"
    )))
}
