use crate::shell::Shell;
use crate::tmux;
use crate::zoxide;
use anyhow::Result;
use std::path::PathBuf;

pub struct ConnectOptions {
    pub tokens: Vec<String>,
    pub dir: Option<PathBuf>,
}

pub struct ParsedLine {
    pub name: String,
    pub path: Option<PathBuf>,
}

pub fn parse_connect_line(tokens: &[String]) -> ParsedLine {
    // Reassemble full line to robustly strip icons and parse path
    let mut s = tokens.join(" ").trim().to_string();

    // Strip leading icon like "[t] " or "[z] " if present
    if let Some(pos) = s.find(']')
        && s.starts_with('[')
        && pos < s.len()
    {
        let after = &s[pos + 1..];
        let after = after.strip_prefix(' ').unwrap_or(after);
        s = after.to_string();
    }
    // Strip fancy icons ( or ) with one or two spaces after
    if let Some(after) = s.strip_prefix("  ") {
        s = after.to_string();
    } else if let Some(after) = s.strip_prefix("  ") {
        s = after.to_string();
    } else if let Some(after) = s.strip_prefix(" ") {
        s = after.to_string();
    } else if let Some(after) = s.strip_prefix(" ") {
        s = after.to_string();
    }

    // Split and find the first absolute path token
    let parts: Vec<&str> = s.split_whitespace().collect();
    let mut split_idx: Option<usize> = None;
    for (i, part) in parts.iter().enumerate() {
        if part.starts_with('/') {
            split_idx = Some(i);
            break;
        }
    }

    let (name_str, path_opt) = match split_idx {
        Some(i) => {
            let name = parts[..i].join(" ");
            let path = parts[i..].join(" ");
            (name, Some(PathBuf::from(path)))
        }
        None => (s.clone(), None),
    };

    // If no explicit name but we have a path, derive name from basename
    let mut name = normalize_name(&name_str);
    if (name.is_empty() || name == "-")
        && path_opt.is_some()
        && let Some(base) = path_opt
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().to_string())
    {
        name = normalize_name(&base);
    }

    ParsedLine {
        name,
        path: path_opt,
    }
}

fn normalize_name(s: &str) -> String {
    let trimmed = s.trim();
    let mut out = String::with_capacity(trimmed.len());
    let mut last_dash = false;
    for ch in trimmed.chars() {
        let c = match ch {
            ':' | '#' => '-',
            c if c.is_whitespace() => ' ',
            _ => ch,
        };
        if c.is_whitespace() {
            if !last_dash {
                out.push('-');
                last_dash = true;
            }
        } else {
            out.push(c);
            last_dash = false;
        }
    }
    out.trim_matches('-').to_string()
}

pub fn run_connect<S: Shell>(sh: &S, opts: ConnectOptions) -> Result<()> {
    let ParsedLine { name, path } = parse_connect_line(&opts.tokens);
    if name.is_empty() {
        anyhow::bail!("empty session name");
    }

    if tmux::has_session(sh, &name).unwrap_or(false) {
        tmux::attach_or_switch(sh, &name)?;
        return Ok(());
    }

    // pick dir: --dir > parsed path > zoxide best match > $HOME
    let dir = if let Some(d) = &opts.dir {
        d.clone()
    } else if let Some(p) = path {
        p
    } else if let Some(p) = zoxide::best_match_dir(sh, &name) {
        p
    } else {
        dirs_next::home_dir().unwrap_or_else(|| PathBuf::from("."))
    };

    tmux::create_session(sh, &name, &dir.to_string_lossy())?;
    tmux::attach_or_switch(sh, &name)?;
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
    fn parse_prefixes_and_path() {
        let p = parse_connect_line(&[
            "[t]".to_string(),
            "my".to_string(),
            "session".to_string(),
            "/work/x".to_string(),
        ]);
        assert_eq!(p.name, "my-session");
        assert_eq!(p.path.as_deref(), Some(std::path::Path::new("/work/x")));

        let p2 = parse_connect_line(&[
            "".to_string(),
            " ".to_string(),
            "api".to_string(),
            "/srv/api".to_string(),
        ]);
        assert_eq!(p2.name, "api");
        assert_eq!(p2.path.as_deref(), Some(std::path::Path::new("/srv/api")));
    }

    #[test]
    fn parse_name_from_basename_when_missing() {
        let p = parse_connect_line(&["/a/b/c".to_string()]);
        assert_eq!(p.name, "c");
        assert_eq!(p.path.as_deref(), Some(std::path::Path::new("/a/b/c")));
    }

    #[test]
    fn connect_attaches_if_exists_switch_inside_tmux() -> Result<()> {
        // tmux present, session exists
        let sh = MockShell::default()
            .status("tmux", &["has-session", "-t", "web"], true)
            .with_env("TMUX", "1");
        run_connect(
            &sh,
            ConnectOptions {
                tokens: vec!["web".into()],
                dir: None,
            },
        )?;
        Ok(())
    }

    #[test]
    fn connect_creates_with_dir_then_attaches() -> Result<()> {
        let sh = MockShell::default().status("tmux", &["has-session", "-t", "web"], false);
        run_connect(
            &sh,
            ConnectOptions {
                tokens: vec!["web".into()],
                dir: Some(PathBuf::from("/x")),
            },
        )?;
        Ok(())
    }
}
