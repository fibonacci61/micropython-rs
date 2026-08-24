use std::path::PathBuf;

use anyhow::{Context, anyhow};
use micropython_manifest::{find_manifest_from, parse_manifest};

pub fn generate<T: Into<String>>(wrapper: T) -> anyhow::Result<()> {
    let out_dir =
        PathBuf::from(std::env::var_os("OUT_DIR").ok_or(anyhow!("`OUT_DIR` is not set"))?);
    let cargo_manifest_dir = PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").ok_or(anyhow!("`CARGO_MANIFEST_DIR` is not set"))?,
    );

    let manifest_paths = find_manifest_from(PathBuf::from(cargo_manifest_dir))?;
    let manifest = parse_manifest(&manifest_paths.path)?;

    let port_dir = manifest.port.path_canonicalized(&manifest_paths.dir)?;
    let mp_dir = manifest
        .micropython
        .path_canonicalized(&manifest_paths.dir)?;

    let py_dir = mp_dir.join("py");
    let py_dir_escaped = regex::escape(
        py_dir
            .to_str()
            .ok_or(anyhow!("micropython path is not valid UTF-8"))?,
    );

    let bindings = bindgen::builder()
        .header(wrapper)
        .use_core()
        .wrap_unsafe_ops(true)
        .allowlist_file(format!(r"{py_dir_escaped}/.*\.h",))
        .clang_arg("-I")
        .clang_arg(
            mp_dir
                .into_string()
                .map_err(|_| anyhow!("micropython path is not valid UTF-8"))?,
        )
        .clang_arg("-I")
        .clang_arg(
            port_dir
                .into_string()
                .map_err(|_| anyhow!("port path is not valid UTF-8"))?,
        )
        .clang_arg("-I")
        .clang_arg(
            manifest_paths
                .dir
                .join("micropython-rs/generated")
                .into_string()
                .map_err(|_| anyhow!("couldn't decode manifest path as UTF-8"))?,
        )
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .context("couldn't generate bindings")?;
    bindings.write_to_file(out_dir.join("bindings.rs"))?;

    Ok(())
}
