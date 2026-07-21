use crate::schema::Config;
use std::path::{Path, PathBuf};

pub const CONFIG_FILE_NAME: &str = "readme.toml";

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("reading {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("writing {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("parsing {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("serializing config: {0}")]
    Serialize(#[from] toml::ser::Error),
}

pub fn find_config() -> Option<PathBuf> {
    find_config_from(&std::env::current_dir().ok()?)
}

pub fn find_config_from(start: &Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        let candidate = dir.join(CONFIG_FILE_NAME);
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

pub fn load(path: &Path) -> Result<Config, ConfigError> {
    let text = std::fs::read_to_string(path).map_err(|source| ConfigError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    toml::from_str(&text).map_err(|source| ConfigError::Parse {
        path: path.to_path_buf(),
        source,
    })
}

pub fn save(path: &Path, config: &Config) -> Result<(), ConfigError> {
    let text = toml::to_string_pretty(config)?;
    std::fs::write(path, text).map_err(|source| ConfigError::Write {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defaults::default_config;

    #[test]
    fn round_trips_default_config() {
        let dir = std::env::temp_dir().join(format!("readme-stuff-config-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(CONFIG_FILE_NAME);

        let original = default_config();
        save(&path, &original).expect("save should succeed");
        let loaded = load(&path).expect("load should succeed");

        assert_eq!(original, loaded);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn find_config_walks_up_parents() {
        let root = std::env::temp_dir().join(format!("readme-stuff-config-find-{}", std::process::id()));
        let nested = root.join("a").join("b");
        std::fs::create_dir_all(&nested).unwrap();
        save(&root.join(CONFIG_FILE_NAME), &default_config()).unwrap();

        let found = find_config_from(&nested).expect("should find config in ancestor dir");
        assert_eq!(found, root.join(CONFIG_FILE_NAME));

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn find_config_returns_none_when_absent() {
        let dir = std::env::temp_dir().join(format!("readme-stuff-config-absent-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(find_config_from(&dir), None);
        std::fs::remove_dir_all(&dir).ok();
    }
}
