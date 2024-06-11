use std::{collections::HashMap, sync::OnceLock};

#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize)]
pub(crate) enum Action {
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
    pub(crate) bindings: HashMap<crokey::KeyCombination, Action>,
}

pub(crate) fn bindings() -> &'static HashMap<crokey::KeyCombination, Action> {
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
        let bindings = bindings();
        assert_eq!(bindings.len(), 14);
    }
}
