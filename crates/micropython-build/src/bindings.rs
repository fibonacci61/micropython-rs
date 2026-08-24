use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow};

/// Generate MicroPython FFI bindings using `bindgen`.
pub fn generate<T: Into<String>>(
    wrapper: T,
    manifest_dir: &Path,
    mp_dir: &Path,
    port_dir: &Path,
) -> anyhow::Result<()> {
    let out_dir =
        PathBuf::from(std::env::var_os("OUT_DIR").ok_or(anyhow!("`OUT_DIR` is not set"))?);

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
                .to_str()
                .ok_or(anyhow!("micropython path is not valid UTF-8"))?,
        )
        .clang_arg("-I")
        .clang_arg(
            port_dir
                .to_str()
                .ok_or(anyhow!("port path is not valid UTF-8"))?,
        )
        .clang_arg("-I")
        .clang_arg(
            manifest_dir
                .join("micropython-rs/generated")
                .into_string()
                .map_err(|_| anyhow!("manifest path is not valid UTF-8"))?,
        )
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .context("couldn't generate bindings")?;
    bindings.write_to_file(out_dir.join("bindings.rs"))?;

    Ok(())
}
