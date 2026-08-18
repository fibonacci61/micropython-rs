mod depfile;

use std::{
    collections::HashMap,
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, bail};
use blake3::{Hash, Hasher};
use regex::bytes::Regex;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use crate::generate::{ScanItem, c::depfile::Depfile, cache_path};

#[derive(Serialize, Deserialize)]
pub struct CCacheItem {
    pub src_path: PathBuf,
    pub context_hash: String,
    pub deps: HashMap<PathBuf, String>,
    pub qstrs: Vec<String>,
    pub moduledefs: Vec<String>,
    pub root_pointers: Vec<String>,
}

fn get_file_hash(path: &Path, hashes: &mut HashMap<PathBuf, Hash>) -> anyhow::Result<Hash> {
    if let Some(hash) = hashes.get(path) {
        return Ok(*hash);
    }

    let hash = match std::fs::read(path) {
        Ok(v) => blake3::hash(&v),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => blake3::hash(&[]),
        Err(e) => return Err(e).context(format!("couldn't read `{}`", path.display())),
    };
    hashes.insert(PathBuf::from(path), hash);

    Ok(hash)
}

pub struct Regexps {
    qstr_re: Regex,
    moduledef_re: Regex,
    root_pointer_re: Regex,
}

impl Regexps {
    pub fn new() -> Self {
        let qstr_re = Regex::new(r#"MP_QSTR_([a-zA-Z_][a-zA-Z0-9_]*)"#).unwrap();
        let moduledef_re = Regex::new(
            r#"(?:MP_REGISTER_MODULE|MP_REGISTER_EXTENSIBLE_MODULE|MP_REGISTER_MODULE_DELEGATION)\(.*?,\s*.*?\);"#,
        ).unwrap();
        let root_pointer_re = Regex::new(r#"MP_REGISTER_ROOT_POINTER\((.*?)\);"#).unwrap();

        Self {
            qstr_re,
            moduledef_re,
            root_pointer_re,
        }
    }
}

pub struct PreprocessorContext {
    args: Vec<OsString>,
}

impl PreprocessorContext {
    pub fn new(src_path: PathBuf, port_dir: PathBuf, mp_dir: PathBuf, header_dir: PathBuf) -> Self {
        Self {
            args: vec![
                OsString::from("-E"),
                OsString::from("-I"),
                port_dir.into_os_string(),
                OsString::from("-I"),
                mp_dir.into_os_string(),
                OsString::from("-I"),
                header_dir.into_os_string(),
                OsString::from("-DNO_QSTR"),
                src_path.into_os_string(),
            ],
        }
    }

    pub fn clang_command(&self) -> Command {
        let mut cmd = Command::new("clang");
        cmd.args(&self.args);
        cmd
    }

    pub fn blake3_hash(&self, hasher: &mut Hasher) {
        self.args.iter().for_each(|arg| {
            hasher.update(arg.as_encoded_bytes());
        });
    }
}

pub fn scan_c(
    pp_context: &PreprocessorContext,
    regexps: &Regexps,
) -> anyhow::Result<(ScanItem, Depfile)> {
    let depfile_path = NamedTempFile::new()
        .context("couldn't create temporary file")?
        .into_temp_path();

    let output = pp_context
        .clang_command()
        .arg("-MMD")
        .arg("-MF")
        .arg(&depfile_path)
        .output()
        .context("couldn't spawn `clang`")?;

    if !output.status.success() {
        bail!(
            "`clang` failed [{}]: {}",
            output.status,
            String::from_utf8_lossy_owned(output.stderr)
        );
    }

    let depfile =
        depfile::parse(&std::fs::read(depfile_path).context("couldn't read temporary file")?)
            .context("couldn't parse clang depfile")?;

    let preprocessed_src = output.stdout;
    let mut qstrs = Vec::new();
    let mut moduledefs = Vec::new();
    let mut root_pointers = Vec::new();

    for qstr_cap in regexps.qstr_re.captures_iter(&preprocessed_src) {
        qstrs.push(String::from_utf8(qstr_cap[1].to_vec()).with_context(|| {
            format!(
                "couldn't decode qstr `{}` as UTF-8",
                String::from_utf8_lossy(&qstr_cap[1])
            )
        })?);
    }

    for moduledef_cap in regexps.moduledef_re.captures_iter(&preprocessed_src) {
        moduledefs.push(
            String::from_utf8(moduledef_cap[0].to_vec()).with_context(|| {
                format!(
                    "couldn't decode moduledef `{}` as UTF-8",
                    String::from_utf8_lossy(&moduledef_cap[0])
                )
            })?,
        );
    }

    for root_pointer_cap in regexps.root_pointer_re.captures_iter(&preprocessed_src) {
        root_pointers.push(
            String::from_utf8(root_pointer_cap[1].to_vec()).with_context(|| {
                format!(
                    "couldn't decode root pointer `{}` as UTF-8",
                    String::from_utf8_lossy(&root_pointer_cap[0])
                )
            })?,
        );
    }

    Ok((
        ScanItem {
            qstrs,
            moduledefs,
            root_pointers,
        },
        depfile,
    ))
}

pub fn cache_c(
    item: ScanItem,
    depfile: Depfile,
    src_path: PathBuf,
    pp_context: &PreprocessorContext,
    cache_path: &Path,
    hashes: &mut HashMap<PathBuf, Hash>,
) -> anyhow::Result<ScanItem> {
    let context_hash = {
        let mut hasher = Hasher::new();
        pp_context.blake3_hash(&mut hasher);
        hasher.finalize()
    };

    let mut deps = HashMap::new();
    for dep in depfile.dependencies {
        let hash = get_file_hash(&dep, hashes)?;
        deps.insert(dep, hash.to_hex().to_string());
    }

    let cache_item = CCacheItem {
        src_path,
        context_hash: context_hash.to_string(),
        deps,
        // temporarily moving values out of `item`
        qstrs: item.qstrs,
        moduledefs: item.moduledefs,
        root_pointers: item.root_pointers,
    };

    let cache_contents = serde_json::to_vec(&cache_item).with_context(|| {
        format!(
            "couldn't serialize cache for `{}`",
            cache_item.src_path.display()
        )
    })?;
    std::fs::write(cache_path, cache_contents)
        .with_context(|| format!("couldn't write cache `{}`", cache_path.display()))?;

    // moving back
    Ok(ScanItem {
        qstrs: cache_item.qstrs,
        moduledefs: cache_item.moduledefs,
        root_pointers: cache_item.root_pointers,
    })
}

pub fn is_cache_valid(
    cache: &CCacheItem,
    pp_context: &PreprocessorContext,
    hashes: &mut HashMap<PathBuf, Hash>,
) -> anyhow::Result<bool> {
    for (dep_path, dep_cached_hash) in cache.deps.iter() {
        let dep_cached_hash =
            Hash::from_hex(dep_cached_hash).context("couldn't decode dep hash")?;
        let dep_hash = get_file_hash(dep_path, hashes)?;

        if dep_cached_hash != dep_hash {
            return Ok(false);
        }
    }

    let context_hash = {
        let mut hasher = Hasher::new();
        pp_context.blake3_hash(&mut hasher);
        hasher.finalize()
    };
    let cached_context_hash =
        Hash::from_hex(&cache.context_hash).context("couldn't decode context hash")?;

    if cached_context_hash != context_hash {
        return Ok(false);
    }

    Ok(true)
}

pub fn scan_c_cached(
    src_path: PathBuf,
    port_dir: PathBuf,
    mp_dir: PathBuf,
    header_dir: PathBuf,
    regexps: &Regexps,
    cache_dir: &Path,
    cache_miss: &mut bool,
    hashes: &mut HashMap<PathBuf, Hash>,
) -> anyhow::Result<ScanItem> {
    let cache_path = cache_path(cache_dir, &src_path);
    let cache = match std::fs::read(&cache_path) {
        Ok(v) => Some(
            serde_json::from_slice::<CCacheItem>(&v)
                .with_context(|| format!("couldn't parse cache `{}`", cache_path.display()))?,
        ),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => {
            return Err(e)
                .with_context(|| format!("couldn't read cache `{}`", cache_path.display()));
        }
    };

    let pp_context = PreprocessorContext::new(src_path.clone(), port_dir, mp_dir, header_dir);
    let item = if let Some(cache) = cache
        && is_cache_valid(&cache, &pp_context, hashes)?
    {
        ScanItem {
            qstrs: cache.qstrs,
            moduledefs: cache.moduledefs,
            root_pointers: cache.root_pointers,
        }
    } else {
        *cache_miss = true;
        let (item, depfile) = scan_c(&pp_context, regexps)
            .with_context(|| format!("couldn't scan C source `{}`", src_path.display()))?;
        cache_c(item, depfile, src_path, &pp_context, &cache_path, hashes)?
    };

    Ok(item)
}
