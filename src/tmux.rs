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

/// Best-effort detection of the currently active/attached session name.
/// Returns None if not determinable.
pub fn active_session<S: Shell>(sh: &S) -> Option<String> {
    // If inside tmux, display-message yields the current session.
    let disp = sh
        .run("tmux", &["display-message", "-p", "-F", "#S"])
        .unwrap_or_default();
    let name = disp.lines().next().map(|s| s.trim()).unwrap_or("");
    if !name.is_empty() {
        return Some(name.to_string());
    }

    // Fallback: scan sessions for an attached one.
    let out = sh
        .run(
            "tmux",
            &["list-sessions", "-F", "#{?session_attached,1,0}\t#S"],
        )
        .unwrap_or_default();
    let mut attached: Vec<String> = out
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(2, '\t');
            match (parts.next(), parts.next()) {
                (Some("1"), Some(n)) => {
                    let n = n.trim();
                    if n.is_empty() {
                        None
                    } else {
                        Some(n.to_string())
                    }
                }
                _ => None,
            }
        })
        .collect();
    attached.sort();
    attached.into_iter().next()
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
        // Attach must run with a TTY; inherit stdio so tmux sees a terminal
        sh.run_tty("tmux", &["attach", "-t", name])?;
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
        fn run_tty(&self, _program: &str, _args: &[&str]) -> Result<()> {
            Ok(())
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
            fn run_tty(&self, _p: &str, _a: &[&str]) -> Result<()> {
                Ok(())
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
