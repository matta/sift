use std::collections::HashMap;
use std::sync::OnceLock;

use crokey::KeyCombination;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub(crate) enum Command {
    Edit,
    Toggle,
    Snooze,
    Next,
    Previous,
    MoveUp,
    MoveDown,
    Add,
    Delete,
    Quit,
}

#[derive(serde::Deserialize)]
pub(crate) struct Config {
    pub(crate) bindings: HashMap<KeyCombination, Command>,
}

pub(crate) fn default_bindings() -> &'static HashMap<KeyCombination, Command> {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    &CONFIG
        .get_or_init(|| {
            let toml = include_str!("keys.toml");
            toml::from_str(toml).unwrap()
        })
        .bindings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let bindings = default_bindings();
        assert_eq!(bindings.len(), 14);
    }
}
