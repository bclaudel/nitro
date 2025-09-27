use crate::shell::Shell;
use crate::tmux;
use crate::zoxide;
use anyhow::Result;

const ICON_TMUX: &str = "";
const ICON_ZOX: &str = "";
const COLOR_MAGENTA: &str = "\u{001b}[35m"; // tmux (icons)
const COLOR_BLUE: &str = "\u{001b}[34m"; // zoxide (icons)
const COLOR_ASCII_T: &str = "\u{001b}[35m"; // tmux [t]
const COLOR_ASCII_Z: &str = "\u{001b}[34m"; // zoxide [z]
const COLOR_RESET: &str = "\u{001b}[0m";

pub struct ListOptions {
    pub include_tmux: bool,
    pub include_zox: bool,
    pub z_limit: Option<usize>,
    pub icons: bool,
    pub no_color: bool,
}

fn colorize(enabled: bool, color: &str, s: &str) -> String {
    if enabled {
        format!("{}{}{}", color, s, COLOR_RESET)
    } else {
        s.to_string()
    }
}

pub fn build_list_lines<S: Shell>(sh: &S, opts: &ListOptions) -> Result<Vec<String>> {
    let mut lines = Vec::new();

    // Color policy: color enabled by default; disable with --no-color only
    // For ASCII mode: color prefixes; for icon mode: color glyphs
    let want_color = !opts.no_color;

    if opts.include_tmux
        && let Ok(mut sessions) = tmux::list_sessions(sh)
    {
        for s in sessions.drain(..) {
            if opts.icons {
                let icon = colorize(want_color, COLOR_MAGENTA, ICON_TMUX);
                lines.push(format!("{}  {}", icon, s));
            } else {
                let prefix = colorize(want_color, COLOR_ASCII_T, "[t]");
                lines.push(format!("{} {}", prefix, s));
            }
        }
    }

    if opts.include_zox
        && let Ok(mut items) = zoxide::list_all(sh)
    {
        if let Some(n) = opts.z_limit {
            items.truncate(n);
        }
        for it in items.into_iter() {
            if opts.icons {
                let icon = colorize(want_color, COLOR_BLUE, ICON_ZOX);
                lines.push(format!("{}  {}", icon, it.path.display()));
            } else {
                let prefix = colorize(want_color, COLOR_ASCII_Z, "[z]");
                lines.push(format!("{} {}", prefix, it.path.display()));
            }
        }
    }

    Ok(lines)
}

pub fn run_list<S: Shell>(sh: &S, opts: ListOptions) -> Result<()> {
    for line in build_list_lines(sh, &opts)? {
        println!("{}", line);
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
        map: HashMap<(String, Vec<String>), String>,
        env: HashMap<String, String>,
    }

    impl MockShell {
        fn with(mut self, program: &str, args: &[&str], output: &str) -> Self {
            self.map.insert(
                (program.into(), args.iter().map(|s| s.to_string()).collect()),
                output.into(),
            );
            self
        }
        #[allow(dead_code)]
        fn with_env(mut self, k: &str, v: &str) -> Self {
            self.env.insert(k.into(), v.into());
            self
        }
    }

    impl crate::shell::Shell for MockShell {
        fn run(&self, program: &str, args: &[&str]) -> Result<String> {
            Ok(self
                .map
                .get(&(program.into(), args.iter().map(|s| s.to_string()).collect()))
                .cloned()
                .unwrap_or_default())
        }
        fn run_status(&self, _program: &str, _args: &[&str]) -> Result<bool> {
            Ok(true)
        }
        fn env_var(&self, key: &str) -> Option<String> {
            self.env.get(key).cloned()
        }
    }

    #[test]
    fn list_tmux_and_zoxide_ascii_and_limit() -> Result<()> {
        let sh = MockShell::default()
            .with("tmux", &["list-sessions", "-F", "#S"], "b\na\n")
            .with(
                "zoxide",
                &["query", "-l"],
                "/home/u/one\n/home/u/two\n/home/u/three\n",
            );

        let opts = ListOptions {
            include_tmux: true,
            include_zox: true,
            z_limit: Some(2),
            icons: false,
            no_color: true,
        };
        let lines = build_list_lines(&sh, &opts)?;
        assert_eq!(
            lines,
            vec![
                String::from("[t] a"),
                String::from("[t] b"),
                String::from("[z] /home/u/one"),
                String::from("[z] /home/u/two"),
            ]
        );
        Ok(())
    }

    fn opts_ascii(no_color: bool) -> ListOptions {
        ListOptions {
            include_tmux: true,
            include_zox: true,
            z_limit: None,
            icons: false,
            no_color,
        }
    }

    fn opts_icons(no_color: bool) -> ListOptions {
        ListOptions {
            include_tmux: true,
            include_zox: true,
            z_limit: None,
            icons: true,
            no_color,
        }
    }

    #[test]
    fn list_icons_with_color() -> Result<()> {
        let sh = MockShell::default()
            .with("tmux", &["list-sessions", "-F", "#S"], "x\n")
            .with("zoxide", &["query", "-l"], "/a/b\n");
        let lines = build_list_lines(&sh, &opts_icons(false))?;
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("\u{001b}["));
        assert!(lines[0].contains("x"));
        assert!(lines[1].contains("\u{001b}["));
        assert!(lines[1].contains("/a/b"));
        Ok(())
    }

    #[test]
    fn list_graceful_empty() -> Result<()> {
        let sh = MockShell::default()
            .with("tmux", &["list-sessions", "-F", "#S"], "\n\n")
            .with("zoxide", &["query", "-l"], "\n");
        let lines = build_list_lines(&sh, &opts_ascii(true))?;
        assert!(lines.is_empty());
        Ok(())
    }
}
