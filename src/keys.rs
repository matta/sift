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
    Undo,
    Redo,
    Quit,
}

#[derive(serde::Deserialize)]
pub(crate) struct Config {
    #[allow(dead_code)]
    pub(crate) bindings: HashMap<KeyCombination, Command>,
}

#[allow(dead_code)]
pub(crate) fn default_bindings() -> &'static HashMap<KeyCombination, Command> {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    &CONFIG
        .get_or_init(|| {
            let toml = include_str!("keys.toml");
            toml::from_str(toml).expect("failed to parse keys.toml")
        })
        .bindings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let bindings = default_bindings();
        assert_eq!(bindings.len(), 16);
    }
}
