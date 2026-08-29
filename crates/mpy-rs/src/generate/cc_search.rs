use std::{env::VarError, ffi::OsStr};

use anyhow::{Context, anyhow, bail};

#[derive(Debug)]
pub enum RawCcSource {
    CliArg,
    CcTargetEnvVar,
    CcTargetUnderscoredEnvVar,
    CcEnvVar,
    CcFallback,
}

pub struct RawCc {
    pub s: String,
    pub source: RawCcSource,
}

#[derive(Debug)]
pub struct Cc<'r> {
    pub program: &'r OsStr,
    pub args: Vec<&'r OsStr>,
}

fn get_env_var(key: impl AsRef<str>) -> anyhow::Result<Option<String>> {
    match std::env::var(key.as_ref()) {
        Ok(v) => Ok(Some(v)),
        Err(VarError::NotPresent) => Ok(None),
        Err(VarError::NotUnicode(_)) => {
            bail!(
                "value of environment variable `{}` contains invalid UTF-8",
                key.as_ref()
            )
        }
    }
}

pub fn get_raw_cc(cc: Option<String>, target: Option<String>) -> anyhow::Result<RawCc> {
    if let Some(cc) = cc {
        return Ok(RawCc {
            s: cc,
            source: RawCcSource::CliArg,
        });
    }

    if let Some(target) = target {
        if let Some(cc) = get_env_var(format!("CC_{target}"))? {
            return Ok(RawCc {
                s: cc,
                source: RawCcSource::CcTargetEnvVar,
            });
        }

        let underscored = target.replace(['-', '.'], "_");
        if let Some(cc) = get_env_var(format!("CC_{underscored}"))? {
            return Ok(RawCc {
                s: cc,
                source: RawCcSource::CcTargetUnderscoredEnvVar,
            });
        }
    }

    if let Some(cc) = get_env_var("CC")? {
        return Ok(RawCc {
            s: cc,
            source: RawCcSource::CcEnvVar,
        });
    }

    Ok(RawCc {
        s: "cc".to_string(),
        source: RawCcSource::CcFallback,
    })
}

pub fn get_cc<'r>(raw_cc: &'r RawCc) -> anyhow::Result<Cc<'r>> {
    let raw_trimmed = raw_cc.s.trim();
    if std::fs::exists(raw_trimmed)
        .with_context(|| format!("couldn't check if C compiler `{raw_trimmed}` exists"))?
    {
        Ok(Cc {
            program: OsStr::new(raw_trimmed),
            args: vec![],
        })
    } else {
        let mut split = raw_trimmed.split(' ');
        let program = split.next().ok_or(anyhow!(
            "C compiler path is empty; sourced from `{:?}`",
            raw_cc.source,
        ))?;

        Ok(Cc {
            program: OsStr::new(program),
            args: split.map(|arg| OsStr::new(arg)).collect(),
        })
    }
}
