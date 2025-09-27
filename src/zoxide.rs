use anyhow::Result;
use std::path::PathBuf;

pub struct ZItem {
    pub path: PathBuf,
}

pub fn list_all<S: crate::shell::Shell>(sh: &S) -> Result<Vec<ZItem>> {
    let out = sh.run("zoxide", &["query", "-l"]).unwrap_or_default();
    let mut items = Vec::new();
    for line in out.lines() {
        let p = line.trim();
        if p.is_empty() {
            continue;
        }
        let path = PathBuf::from(p);
        items.push(ZItem { path });
    }
    Ok(items)
}

pub fn best_match_dir<S: crate::shell::Shell>(sh: &S, query: &str) -> Option<PathBuf> {
    // Let zoxide decide best match
    match sh.run("zoxide", &["query", query]) {
        Ok(s) => {
            let p = s
                .lines()
                .next()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())?;
            Some(PathBuf::from(p))
        }
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn list_all_parses_and_ignores_empty() -> Result<()> {
        struct RShell;
        impl crate::shell::Shell for RShell {
            fn run(&self, _p: &str, _a: &[&str]) -> Result<String> {
                Ok("/a\n\n /b \n".into())
            }
            fn run_status(&self, _p: &str, _a: &[&str]) -> Result<bool> {
                Ok(true)
            }
            fn run_tty(&self, _p: &str, _a: &[&str]) -> Result<()> {
                Ok(())
            }
            fn env_var(&self, _k: &str) -> Option<String> {
                None
            }
        }
        let v = list_all(&RShell)?;
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].path, PathBuf::from("/a"));
        assert_eq!(v[1].path, PathBuf::from("/b"));
        Ok(())
    }

    #[test]
    fn best_match_ok_and_err() {
        struct OkShell;
        impl crate::shell::Shell for OkShell {
            fn run(&self, _p: &str, _a: &[&str]) -> Result<String> {
                Ok("/match\n".into())
            }
            fn run_status(&self, _p: &str, _a: &[&str]) -> Result<bool> {
                Ok(true)
            }
            fn run_tty(&self, _p: &str, _a: &[&str]) -> Result<()> {
                Ok(())
            }
            fn env_var(&self, _k: &str) -> Option<String> {
                None
            }
        }
        struct ErrShell;
        impl crate::shell::Shell for ErrShell {
            fn run(&self, _p: &str, _a: &[&str]) -> Result<String> {
                anyhow::bail!("boom")
            }
            fn run_status(&self, _p: &str, _a: &[&str]) -> Result<bool> {
                Ok(true)
            }
            fn run_tty(&self, _p: &str, _a: &[&str]) -> Result<()> {
                Ok(())
            }
            fn env_var(&self, _k: &str) -> Option<String> {
                None
            }
        }
        assert_eq!(best_match_dir(&OkShell, "x"), Some(PathBuf::from("/match")));
        assert_eq!(best_match_dir(&ErrShell, "x"), None);
    }
}
