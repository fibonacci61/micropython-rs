pub mod c;
pub mod rust;

use std::process::Command;
use std::{ffi::OsStr, path::Path, path::PathBuf};

use anyhow::Context;
use anyhow::bail;
use walkdir::WalkDir;

use crate::generate::rust::scan_rust_cached;
use crate::manifest::{Crate, MicroPython, find_manifest, parse_manifest};

pub fn gen_version_header(py_dir: &Path, genhdr_dir: &Path) -> anyhow::Result<()> {
    let makeversionhdr_path = py_dir.join("makeversionhdr.py");
    let output = Command::new("python3")
        .arg(&makeversionhdr_path)
        .arg(genhdr_dir.join("mpversion.h"))
        .output()
        .context("couldn't execute python3")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "`{}` failed with status {}: {}",
            makeversionhdr_path.display(),
            output.status,
            stderr.trim()
        );
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSearch {
    pub c_srcs: Vec<PathBuf>,
    pub rust_srcs: Vec<PathBuf>,
}

pub fn search_project(
    manifest_dir: &Path,
    py_dir: &Path,
    crates: &[Crate],
) -> anyhow::Result<ProjectSearch> {
    assert!(manifest_dir.is_absolute());
    assert!(py_dir.is_absolute());

    let mut c_srcs = Vec::new();
    let mut rust_srcs = Vec::new();

    for entry in WalkDir::new(py_dir).max_depth(1).into_iter() {
        let entry = match entry {
            Ok(v) => v,
            Err(e) => {
                return Err(anyhow::Error::from(e)
                    .context(format!("couldn't scan directory `{}`", py_dir.display())));
            }
        };

        if entry.file_type().is_file() && entry.path().extension() == Some(OsStr::new("c")) {
            c_srcs.push(entry.into_path());
        }
    }

    // cr7 suiii
    for cr8 in crates {
        let absolute_crate_path = if cr8.path.is_absolute() {
            PathBuf::from(&cr8.path)
        } else {
            manifest_dir.join(&cr8.path)
        }
        // need to canonicalize to get rid of any "."/".." in cr8.path
        .canonicalize()?;

        let src_dir = absolute_crate_path.join("src");
        for entry in WalkDir::new(&src_dir).into_iter() {
            let entry = match entry {
                Ok(v) => v,
                Err(e) => {
                    return Err(anyhow::Error::from(e).context(format!(
                        "couldn't scan crate `{}`",
                        absolute_crate_path.display()
                    )));
                }
            };

            if entry.file_type().is_file() && entry.path().extension() == Some(OsStr::new("rs")) {
                rust_srcs.push(entry.into_path());
            }
        }
    }

    Ok(ProjectSearch { c_srcs, rust_srcs })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanItem {
    pub qstrs: Vec<String>,
    pub moduledefs: Vec<String>,
    pub root_pointers: Vec<String>,
}

pub fn generate(dir: Option<PathBuf>) -> anyhow::Result<()> {
    let manifest_paths = find_manifest(dir)?;
    let manifest = parse_manifest(&manifest_paths.path)?;
    let Some(MicroPython { path: mp_dir }) = manifest.micropython else {
        bail!(
            "`[micropython]` is required in `{}`",
            manifest_paths.path.display()
        );
    };
    let py_dir = mp_dir.join("py");

    let header_dir = manifest_paths.dir.join("micropython-rs/generated");
    let genhdr_dir = header_dir.join("genhdr");
    // don't need to create `header_dir` first cause of `create_dir_all`
    std::fs::create_dir_all(&genhdr_dir)
        .with_context(|| format!("couldn't create directory `{}`", genhdr_dir.display()))?;

    gen_version_header(&py_dir, &genhdr_dir)?;

    let project_search = search_project(&manifest_paths.dir, &py_dir, &manifest.crates)?;
    let mut items = Vec::new();

    let cache_dir = header_dir.join("cache");
    std::fs::create_dir_all(&cache_dir)
        .with_context(|| format!("couldn't create directory `{}`", cache_dir.display()))?;
    for rust_src in project_search.rust_srcs.iter() {
        items.push(scan_rust_cached(rust_src, &cache_dir)?);
    }

    Ok(())
}
