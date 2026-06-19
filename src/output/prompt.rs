use std::fmt;
use std::io::{self, IsTerminal, Write};

pub fn confirm(prompt: impl fmt::Display, default_yes: bool) -> anyhow::Result<bool> {
    if !io::stdin().is_terminal() {
        anyhow::bail!(
            "Confirmation required for non-interactive input. Run from a terminal or stage the files manually."
        );
    }

    let suffix = if default_yes { " [Y/n] " } else { " [y/N]: " };
    print!("{prompt}{suffix}");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(resolve_confirmation(input.trim(), default_yes))
}

fn resolve_confirmation(input: &str, default_yes: bool) -> bool {
    match input.trim().to_ascii_lowercase().as_str() {
        "y" | "yes" => true,
        "" => default_yes,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_confirmation;

    #[test]
    fn confirmation_accepts_yes_values() {
        assert!(resolve_confirmation("y", false));
        assert!(resolve_confirmation("YES", false));
    }

    #[test]
    fn confirmation_uses_default_for_empty_input() {
        assert!(resolve_confirmation("", true));
        assert!(!resolve_confirmation("", false));
    }

    #[test]
    fn confirmation_rejects_other_values() {
        assert!(!resolve_confirmation("n", true));
        assert!(!resolve_confirmation("anything else", true));
    }
}
