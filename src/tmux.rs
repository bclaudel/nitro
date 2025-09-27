use crate::shell::Shell;
use anyhow::Result;

pub fn list_sessions<S: Shell>(sh: &S) -> Result<Vec<String>> {
    let out = sh
        .run("tmux", &["list-sessions", "-F", "#S"])
        .unwrap_or_default();
    let mut items: Vec<String> = out
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    items.sort();
    Ok(items)
}

pub fn has_session<S: Shell>(sh: &S, name: &str) -> Result<bool> {
    // tmux has-session returns non-zero if not exists
    let ok = sh.run_status("tmux", &["has-session", "-t", name])?;
    Ok(ok)
}

pub fn create_session<S: Shell>(sh: &S, name: &str, dir: &str) -> Result<()> {
    let _ = sh.run("tmux", &["new-session", "-ds", name, "-c", dir])?;
    Ok(())
}

pub fn attach_or_switch<S: Shell>(sh: &S, name: &str) -> Result<()> {
    if sh.env_var("TMUX").is_some() {
        let _ = sh.run("tmux", &["switch-client", "-t", name])?;
    } else {
        let _ = sh.run("tmux", &["attach", "-t", name])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::collections::HashMap;

    #[derive(Default)]
    struct MockShell {
        status: HashMap<(String, Vec<String>), bool>,
        env: HashMap<String, String>,
    }

    impl MockShell {
        fn status(mut self, program: &str, args: &[&str], ok: bool) -> Self {
            self.status.insert(
                (program.into(), args.iter().map(|s| s.to_string()).collect()),
                ok,
            );
            self
        }
        fn with_env(mut self, k: &str, v: &str) -> Self {
            self.env.insert(k.into(), v.into());
            self
        }
    }

    impl crate::shell::Shell for MockShell {
        fn run(&self, _program: &str, _args: &[&str]) -> Result<String> {
            Ok(String::new())
        }
        fn run_status(&self, program: &str, args: &[&str]) -> Result<bool> {
            Ok(*self
                .status
                .get(&(program.into(), args.iter().map(|s| s.to_string()).collect()))
                .unwrap_or(&true))
        }
        fn env_var(&self, key: &str) -> Option<String> {
            self.env.get(key).cloned()
        }
    }

    #[test]
    fn list_sessions_sorts() -> Result<()> {
        struct RShell;
        impl crate::shell::Shell for RShell {
            fn run(&self, _p: &str, _a: &[&str]) -> Result<String> {
                Ok("b\na\n".into())
            }
            fn run_status(&self, _p: &str, _a: &[&str]) -> Result<bool> {
                Ok(true)
            }
            fn env_var(&self, _k: &str) -> Option<String> {
                None
            }
        }
        let v = list_sessions(&RShell)?;
        assert_eq!(v, vec!["a", "b"]);
        Ok(())
    }

    #[test]
    fn has_session_true_false() -> Result<()> {
        let sh_true = MockShell::default().status("tmux", &["has-session", "-t", "x"], true);
        assert!(has_session(&sh_true, "x")?);
        let sh_false = MockShell::default().status("tmux", &["has-session", "-t", "x"], false);
        assert!(!has_session(&sh_false, "x")?);
        Ok(())
    }

    #[test]
    fn attach_or_switch_respects_tmux_env() -> Result<()> {
        let sh_in = MockShell::default().with_env("TMUX", "1");
        attach_or_switch(&sh_in, "name")?;
        let sh_out = MockShell::default();
        attach_or_switch(&sh_out, "name")?;
        Ok(())
    }
}
