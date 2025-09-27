use anyhow::{Context, Result};
use std::process::{Command, Stdio};

pub trait Shell {
    fn run(&self, program: &str, args: &[&str]) -> Result<String>;
    fn run_status(&self, program: &str, args: &[&str]) -> Result<bool>;
    fn env_var(&self, key: &str) -> Option<String>;
}

#[derive(Debug, Default, Clone)]
pub struct RealShell;

impl Shell for RealShell {
    fn run(&self, program: &str, args: &[&str]) -> Result<String> {
        let output = Command::new(program)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .with_context(|| format!("failed to spawn {}", program))?;
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("{} {:?} failed: {}", program, args, err.trim());
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn run_status(&self, program: &str, args: &[&str]) -> Result<bool> {
        let status = Command::new(program)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .with_context(|| format!("failed to spawn {}", program))?;
        Ok(status.success())
    }

    fn env_var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}
