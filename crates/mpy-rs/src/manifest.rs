use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use serde::Deserialize;

const MANIFEST_FILE_NAME: &str = "micropython-rs.toml";

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub port: Option<Port>,
    pub micropython: Option<MicroPython>,
    #[serde(default, rename = "crate")]
    pub crates: Vec<Crate>,
}

#[derive(Debug, Deserialize)]
pub struct Port {
    pub path: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct MicroPython {
    pub path: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct Crate {
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestPaths {
    pub path: PathBuf,
    pub dir: PathBuf,
}

pub fn find_manifest(dir: Option<PathBuf>) -> anyhow::Result<ManifestPaths> {
    let mut dir = dir.unwrap_or_else(|| std::env::current_dir().unwrap());
    assert!(dir.is_absolute());

    loop {
        let path = dir.join(MANIFEST_FILE_NAME);
        if std::fs::exists(&path)
            .with_context(|| format!("couldn't check existence of `{}`", path.display()))?
        {
            return Ok(ManifestPaths { path, dir });
        }

        if !dir.pop() {
            bail!("couldn't find `{MANIFEST_FILE_NAME}`");
        }
    }
}

pub fn parse_manifest(path: &Path) -> anyhow::Result<Manifest> {
    let source =
        std::fs::read(path).with_context(|| format!("couldn't read `{}`", path.display()))?;
    let manifest = toml::from_slice(&source)
        .with_context(|| format!("couldn't parse manifest `{}`", path.display()))?;
    Ok(manifest)
}

impl Port {
    pub fn path_canonicalized(&self, manifest_dir: &Path) -> anyhow::Result<PathBuf> {
        manifest_dir
            .join(&self.path)
            .canonicalize()
            .with_context(|| format!("port directory `{}` not available", self.path.display()))
    }
}

impl MicroPython {
    pub fn path_canonicalized(&self, manifest_dir: &Path) -> anyhow::Result<PathBuf> {
        manifest_dir
            .join(&self.path)
            .canonicalize()
            .with_context(|| {
                format!(
                    "micropython directory `{}` not available",
                    self.path.display()
                )
            })
    }
}
