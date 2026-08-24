pub mod c;
pub mod rust;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::{Command, Stdio};
use std::{ffi::OsStr, path::Path, path::PathBuf};

use anyhow::Context;
use anyhow::bail;
use regex::bytes::Regex;
use walkdir::WalkDir;

use crate::generate::c::{PreprocessorContext, scan_c_cached};
use crate::generate::rust::scan_rust_cached;
use micropython_manifest::{Crate, find_manifest, parse_manifest};

pub fn gen_version_header(py_dir: &Path, genhdr_dir: &Path) -> anyhow::Result<()> {
    let makeversionhdr_path = py_dir.join("makeversionhdr.py");
    let output = Command::new("python3")
        .arg(&makeversionhdr_path)
        .arg(genhdr_dir.join("mpversion.h"))
        .output()
        .context("couldn't execute `python3`")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "`{}` failed [{}]: {}",
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
        let absolute_crate_path = manifest_dir.join(&cr8.path).canonicalize()?;

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

pub fn cache_path(cache_dir: &Path, absolute_path: &Path) -> PathBuf {
    let hash = blake3::hash(absolute_path.as_os_str().as_encoded_bytes());
    cache_dir.join(format!("{}.json", hash.to_hex()))
}

pub fn gen_qstrdefs(
    py_dir: &Path,
    genhdr_dir: &Path,
    port_dir: PathBuf,
    mp_dir: PathBuf,
    header_dir: PathBuf,
    items: &[ScanItem],
) -> anyhow::Result<()> {
    let mut collected_qstrdefs_quoted = tempfile::Builder::new()
        .suffix(".h")
        .tempfile()
        .context("couldn't create temporary file")?;
    let q_re = Regex::new(r"Q\(.*\)").unwrap();

    {
        let qstrdefs_h_path = py_dir.join("qstrdefs.h");
        let qstrdefs_h = File::open(&qstrdefs_h_path)
            .with_context(|| format!("couldn't open `{}`", qstrdefs_h_path.display()))?;
        let reader = BufReader::new(qstrdefs_h);

        for line in reader.lines() {
            let line =
                line.with_context(|| format!("couldn't read `{}`", qstrdefs_h_path.display()))?;

            if q_re.is_match(line.as_bytes()) {
                writeln!(collected_qstrdefs_quoted, "\"{line}\"")
            } else {
                writeln!(collected_qstrdefs_quoted, "{line}")
            }
            .with_context(|| {
                format!(
                    "couldn't write to `{}`",
                    collected_qstrdefs_quoted.path().display()
                )
            })?;
        }
    }

    for qstr in items.iter().flat_map(|item| item.qstrs.iter()) {
        writeln!(collected_qstrdefs_quoted, "\"Q({qstr})\"").with_context(|| {
            format!(
                "couldn't write to `{}`",
                collected_qstrdefs_quoted.path().display()
            )
        })?;
    }

    collected_qstrdefs_quoted.flush()?;

    let pp_context = PreprocessorContext::new(
        collected_qstrdefs_quoted.path().into(),
        port_dir,
        mp_dir,
        header_dir,
    );
    let mut clang = pp_context
        .clang_command()
        .stdout(Stdio::piped())
        .spawn()
        .context("couldn't spawn `clang`")?;

    let clang_stdout = clang.stdout.take().unwrap();
    let clang_reader = BufReader::new(clang_stdout);

    let qstrdefs_preprocessed_h_path = genhdr_dir.join("qstrdefs.preprocessed.h");
    let qstrdefs_preprocessed_h = File::create(&qstrdefs_preprocessed_h_path)
        .with_context(|| format!("couldn't open `{}`", qstrdefs_preprocessed_h_path.display()))?;
    let mut writer = BufWriter::new(qstrdefs_preprocessed_h);

    for line in clang_reader.lines() {
        let line = line?;

        if q_re.is_match(line.as_bytes()) {
            let line = line.strip_circumfix('"', '"').unwrap();
            writeln!(writer, "{line}")
        } else {
            writeln!(writer, "{line}")
        }
        .with_context(|| {
            format!(
                "couldn't write to `{}`",
                qstrdefs_preprocessed_h_path.display()
            )
        })?;
    }

    let status = clang.wait()?;
    if !status.success() {
        bail!("`clang` failed [{status}]");
    }

    let qstrdefs_generated_h_path = genhdr_dir.join("qstrdefs.generated.h");
    let qstrdefs_generated_h = File::create(&qstrdefs_generated_h_path)
        .with_context(|| format!("couldn't open `{}`", qstrdefs_generated_h_path.display()))?;

    let makeqstrdata_path = py_dir.join("makeqstrdata.py");
    let status = Command::new("python3")
        .arg(&makeqstrdata_path)
        .arg(&qstrdefs_preprocessed_h_path)
        .stdout(Stdio::from(qstrdefs_generated_h))
        .status()
        .context("couldn't execute `python3`")?;

    if !status.success() {
        bail!("`{}` failed [{status}]", makeqstrdata_path.display());
    }

    Ok(())
}

pub fn gen_moduledefs(py_dir: &Path, genhdr_dir: &Path, items: &[ScanItem]) -> anyhow::Result<()> {
    let moduledefs_collected_path = genhdr_dir.join("moduledefs.collected");
    let mut moduledefs_collected = File::create(&moduledefs_collected_path)
        .with_context(|| format!("couldn't open `{}`", moduledefs_collected_path.display()))?;

    for moduledef in items.iter().flat_map(|item| item.moduledefs.iter()) {
        writeln!(moduledefs_collected, "{moduledef}").with_context(|| {
            format!(
                "couldn't write to `{}`",
                moduledefs_collected_path.display()
            )
        })?;
    }

    let moduledefs_h_path = genhdr_dir.join("moduledefs.h");
    let moduledefs_h = File::create(&moduledefs_h_path)
        .with_context(|| format!("couldn't open `{}`", moduledefs_h_path.display()))?;

    let makemoduledefs_path = py_dir.join("makemoduledefs.py");
    let status = Command::new("python3")
        .arg(&makemoduledefs_path)
        .arg(moduledefs_collected_path)
        .stdout(Stdio::from(moduledefs_h))
        .status()
        .context("couldn't execute `python3`")?;

    if !status.success() {
        bail!("`python3` failed [{}]", status);
    }

    Ok(())
}

pub fn gen_root_pointers(genhdr_dir: &Path, items: &[ScanItem]) -> anyhow::Result<()> {
    let root_pointers_h_path = genhdr_dir.join("root_pointers.h");
    let mut root_pointers_h = File::create(&root_pointers_h_path)
        .with_context(|| format!("couldn't open `{}`", root_pointers_h_path.display()))?;

    for root_pointer in items.iter().flat_map(|item| item.root_pointers.iter()) {
        writeln!(root_pointers_h, "{root_pointer};")
            .with_context(|| format!("couldn't write to `{}`", root_pointers_h_path.display()))?;
    }

    Ok(())
}

pub fn generate() -> anyhow::Result<()> {
    let manifest_paths = find_manifest()?;
    let manifest = parse_manifest(&manifest_paths.path)?;

    let mp_dir = manifest
        .micropython
        .path_canonicalized(&manifest_paths.dir)?;
    let port_dir = manifest.port.path_canonicalized(&manifest_paths.dir)?;

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
    let mut cache_miss = false;

    let rust_regexps = rust::Regexps::new();
    for rust_src in project_search.rust_srcs.iter() {
        items.push(scan_rust_cached(
            rust_src,
            &rust_regexps,
            &cache_dir,
            &mut cache_miss,
        )?);
    }

    let c_regexps = c::Regexps::new();
    let mut hashes = HashMap::new();

    for c_src in project_search
        .c_srcs
        .into_iter()
        .chain([py_dir.join("mpconfig.h"), port_dir.join("mpconfigport.h")])
    {
        items.push(scan_c_cached(
            c_src,
            port_dir.clone(),
            mp_dir.clone(),
            header_dir.clone(),
            &c_regexps,
            &cache_dir,
            &mut cache_miss,
            &mut hashes,
        )?);
    }

    if !cache_miss {
        return Ok(());
    }

    gen_qstrdefs(&py_dir, &genhdr_dir, port_dir, mp_dir, header_dir, &items)?;
    gen_moduledefs(&py_dir, &genhdr_dir, &items)?;
    gen_root_pointers(&genhdr_dir, &items)?;

    Ok(())
}
