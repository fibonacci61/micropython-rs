use std::path::{Path, PathBuf};

use anyhow::Context;
use blake3::Hash;
use regex::bytes::Regex;
use serde::{Deserialize, Serialize};

use crate::generate::{ScanItem, cache_path};

#[derive(Serialize, Deserialize)]
pub struct RustCacheItem {
    pub src_path: PathBuf,
    pub src_hash: String,
    pub qstrs: Vec<String>,
    pub moduledefs: Vec<String>,
    pub root_pointers: Vec<String>,
}

pub struct Regexps {
    qstr_re: Regex,
    method_ident_re: Regex,
    constant_ident_re: Regex,
}

impl Regexps {
    pub fn new() -> Self {
        let qstr_re = Regex::new(r#"qstr!\(([a-zA-Z_][a-zA-Z0-9_]*)\)"#).unwrap();
        let method_ident_re = Regex::new(
            r#"#\[method.*]\s*(?:#\[stub.*\])?\s*(?:pub\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)"#,
        )
        .unwrap();
        let constant_ident_re = Regex::new(
            r#"#\[constant\]\s*(?:#\[stub.*\])?\s*(?:pub\s+)?const\s+([a-zA-Z_][a-zA-Z0-9_]*)"#,
        )
        .unwrap();

        Self {
            qstr_re,
            method_ident_re,
            constant_ident_re,
        }
    }
}

pub fn scan_rust(contents: &[u8], regexps: &Regexps) -> anyhow::Result<ScanItem> {
    let mut qstrs = Vec::new();
    for cap in regexps
        .qstr_re
        .captures_iter(contents)
        .chain(regexps.method_ident_re.captures_iter(contents))
        .chain(regexps.constant_ident_re.captures_iter(contents))
    {
        qstrs.push(String::from_utf8(cap[1].to_vec()).with_context(|| {
            format!(
                "couldn't decode qstr `{}` as UTF-8",
                String::from_utf8_lossy(&cap[1])
            )
        })?);
    }

    Ok(ScanItem {
        qstrs,
        // TODO: `micropython-rs` currently cannot define modules or root pointers, add sacnning
        // once bindings support exists
        moduledefs: vec![],
        root_pointers: vec![],
    })
}

pub fn cache_rust(
    item: ScanItem,
    cache_path: &Path,
    src_path: PathBuf,
    src_hash: Hash,
) -> anyhow::Result<ScanItem> {
    let cache_item = RustCacheItem {
        src_path,
        src_hash: src_hash.to_hex().to_string(),
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

pub fn is_cache_valid(cache: &RustCacheItem, src_hash: Hash) -> anyhow::Result<bool> {
    let cache_src_hash = Hash::from_hex(&cache.src_hash).context("couldn't decode hash")?;
    Ok(cache_src_hash == src_hash)
}

pub fn scan_rust_cached(
    src_path: &Path,
    regexps: &Regexps,
    cache_dir: &Path,
) -> anyhow::Result<ScanItem> {
    let cache_path = cache_path(cache_dir, src_path);

    let cache = match std::fs::read(&cache_path) {
        Ok(v) => Some(
            serde_json::from_slice::<RustCacheItem>(&v)
                .with_context(|| format!("couldn't parse cache `{}`", cache_path.display()))?,
        ),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => {
            return Err(e).context(format!("couldn't read cache `{}`", cache_path.display()));
        }
    };

    let src_contents = std::fs::read(src_path)
        .with_context(|| format!("couldn't read Rust source `{}`", src_path.display()))?;
    let src_hash = blake3::hash(&src_contents);

    let item = if let Some(cache) = cache
        && is_cache_valid(&cache, src_hash)?
    {
        ScanItem {
            qstrs: cache.qstrs,
            moduledefs: cache.moduledefs,
            root_pointers: cache.root_pointers,
        }
    } else {
        let item = scan_rust(&src_contents, &regexps)
            .with_context(|| format!("couldn't scan Rust source `{}`", src_path.display()))?;
        cache_rust(item, &cache_path, PathBuf::from(src_path), src_hash)?
    };

    Ok(item)
}
