use std::path::Path;

use anyhow::Context;

pub fn is_scanning(manifest_dir: &Path) -> anyhow::Result<bool> {
    let scan_marker_path = manifest_dir.join("micropython-rs/SCANNING");
    std::fs::exists(&scan_marker_path).with_context(|| {
        format!(
            "couldn't check existence of scan marker `{}`",
            scan_marker_path.display()
        )
    })
}

pub fn emit_scan_cfgs(manifest_dir: &Path) -> anyhow::Result<bool> {
    println!("cargo::rustc-check-cfg=cfg(micropython_rs_qstr_scan)");
    let scanning = is_scanning(manifest_dir)?;

    if scanning {
        println!("cargo::rustc-cfg=micropython_rs_qstr_scan");
    }

    Ok(scanning)
}
