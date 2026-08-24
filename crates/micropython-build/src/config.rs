pub mod expression;
pub mod names;
pub mod tail_parser;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::{io::Write, process::Command};

use anyhow::{Context, anyhow, bail};
use micropython_manifest::{find_manifest_from, parse_manifest};
use tempfile::NamedTempFile;

use crate::config::expression::{ConfigValue, EvaluatedValue, evaluate_config};
use crate::config::names::{CONFIG_NAMES, ENUM_CONFIGS};
use crate::config::tail_parser::parse_tail;

const TAIL: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/tail.c"));

pub struct Config {
    definitions: BTreeMap<String, ConfigValue>,
    deps: Vec<PathBuf>,
}

impl Config {
    pub fn definitions(&self) -> &BTreeMap<String, ConfigValue> {
        &self.definitions
    }

    pub fn deps(&self) -> &Vec<PathBuf> {
        &self.deps
    }

    pub fn into_inner(self) -> (BTreeMap<String, ConfigValue>, Vec<PathBuf>) {
        (self.definitions, self.deps)
    }

    pub fn emit_cargo_directives(&self) {
        let checked_values = CONFIG_NAMES
            .iter()
            .map(|name| format!("{name:?}"))
            .collect::<Vec<_>>()
            .join(", ");
        println!("cargo::rustc-check-cfg=cfg(micropython, values({checked_values}))");

        let enum_names = ENUM_CONFIGS
            .iter()
            .flat_map(|(selector, constants)| {
                std::iter::once(*selector).chain(constants.iter().copied())
            })
            .collect::<std::collections::BTreeSet<_>>();

        for (name, value) in self
            .definitions
            .iter()
            .filter(|(name, _)| !enum_names.contains(name.as_str()))
            .filter_map(|(name, value)| value.evaluated.as_ref().ok().map(|value| (name, value)))
        {
            let enabled = match value {
                EvaluatedValue::Integer(value) => *value != 0,
                EvaluatedValue::EmptyDefinition => true,
            };
            if enabled {
                println!("cargo::rustc-cfg=micropython={name:?}");
            }
        }

        for (selector, constants) in ENUM_CONFIGS {
            if let Some(constant) = selected_enum_constant(&self.definitions, selector, constants) {
                println!("cargo::rustc-cfg=micropython={constant:?}");
            }
        }

        for dep in self.deps.iter() {
            println!("cargo::rerun-if-changed={}", dep.display());
        }
    }
}

fn selected_enum_constant<'a>(
    definitions: &BTreeMap<String, ConfigValue>,
    selector: &str,
    constants: &'a [&str],
) -> Option<&'a str> {
    let selected = definitions.get(selector)?.integer()?;
    constants.iter().copied().find(|constant| {
        definitions.get(*constant).and_then(ConfigValue::integer) == Some(selected)
    })
}

pub fn process_mp_config() -> anyhow::Result<Config> {
    let cargo_manifest_dir =
        std::env::var_os("CARGO_MANIFEST_DIR").ok_or(anyhow!("`CARGO_MANIFEST_DIR` is not set"))?;

    let manifest_paths = find_manifest_from(PathBuf::from(cargo_manifest_dir))?;
    let manifest = parse_manifest(&manifest_paths.path)?;

    let port = manifest
        .port
        .ok_or(anyhow!("`[port]` is required by `micropython-build`"))?;
    let port_dir = port.path_canonicalized(&manifest_paths.dir)?;

    let mp = manifest.micropython.ok_or(anyhow!(
        "`[micropython]` is required by `micropython-build`"
    ))?;
    let mp_dir = mp.path_canonicalized(&manifest_paths.dir)?;

    let mut wrapper_file = tempfile::Builder::new()
        .suffix(".c")
        .tempfile()
        .context("couldn't create temporary file")?;
    wrapper_file
        .write_all(TAIL)
        .context("couldn't write to temporary file")?;

    let depfile_path = NamedTempFile::new()
        .context("couldn't crate temporary file")?
        .into_temp_path();

    let mut cmd = Command::new("clang");
    cmd.arg("-E")
        .arg("-MMD")
        .arg("-MF")
        .arg(&depfile_path)
        .arg("-I")
        .arg(&mp_dir)
        .arg("-I")
        .arg(&port_dir)
        .arg(wrapper_file.path());
    let output = cmd.output().context("couldn't execute `clang`")?;

    if !output.status.success() {
        bail!(
            "`{cmd:?}` failed [{}]; stderr:\n{}",
            output.status,
            String::from_utf8_lossy_owned(output.stderr)
        );
    }

    let tail = parse_tail(
        str::from_utf8(&output.stdout).context("couldn't decode `clang` output as UTF-8")?,
    )
    .context("couldn't parse `clang` generated config tail")?;
    let definitions = evaluate_config(tail);

    let depfile = mpy_rs::generate::c::depfile::parse(
        &std::fs::read(depfile_path).context("couldn't read temporary file")?,
    )
    .context("couldn't parse `clang` depfile")?;
    let mut deps = depfile.dependencies;
    // remove wrapper file from deps
    deps.remove(
        deps.iter()
            .position(|dep| dep == wrapper_file.path())
            .unwrap(),
    );

    Ok(Config { definitions, deps })
}
