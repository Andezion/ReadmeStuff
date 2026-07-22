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

fn find_dotenv_from(start: &Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        let candidate = dir.join(".env");
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn load_dotenv_from(path: &Path) {
    let Ok(text) = std::fs::read_to_string(path) else {
        return;
    };
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, val)) = line.split_once('=') {
            if std::env::var(key).is_err() {
                unsafe { std::env::set_var(key, val.trim()) };
            }
        } else if (line.starts_with("ghp_") || line.starts_with("github_pat_"))
            && std::env::var("GITHUB_TOKEN").is_err()
        {
            unsafe { std::env::set_var("GITHUB_TOKEN", line) };
        }
    }
}

pub fn load_dotenv() {
    if let Some(path) = std::env::current_dir()
        .ok()
        .and_then(|d| find_dotenv_from(&d))
    {
        load_dotenv_from(&path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defaults::default_config;

    #[test]
    fn round_trips_default_config() {
        let dir =
            std::env::temp_dir().join(format!("readme-stuff-config-test-{}", std::process::id()));
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
        let root =
            std::env::temp_dir().join(format!("readme-stuff-config-find-{}", std::process::id()));
        let nested = root.join("a").join("b");
        std::fs::create_dir_all(&nested).unwrap();
        save(&root.join(CONFIG_FILE_NAME), &default_config()).unwrap();

        let found = find_config_from(&nested).expect("should find config in ancestor dir");
        assert_eq!(found, root.join(CONFIG_FILE_NAME));

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn find_config_returns_none_when_absent() {
        let dir =
            std::env::temp_dir().join(format!("readme-stuff-config-absent-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(find_config_from(&dir), None);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn find_dotenv_walks_up_parents() {
        let root = std::env::temp_dir().join(format!(
            "readme-stuff-config-dotenv-find-{}",
            std::process::id()
        ));
        let nested = root.join("a").join("b");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(root.join(".env"), "X=1\n").unwrap();

        let found = find_dotenv_from(&nested).expect("should find .env in ancestor dir");
        assert_eq!(found, root.join(".env"));

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn load_dotenv_from_sets_unset_vars_but_never_overrides_existing_ones() {
        let dir = std::env::temp_dir().join(format!(
            "readme-stuff-config-dotenv-load-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(".env");
        std::fs::write(
            &path,
            "READMESTUFF_TEST_UNSET_VAR=from-dotenv\nREADMESTUFF_TEST_ALREADY_SET=from-dotenv\n",
        )
        .unwrap();

        unsafe { std::env::remove_var("READMESTUFF_TEST_UNSET_VAR") };
        unsafe { std::env::set_var("READMESTUFF_TEST_ALREADY_SET", "from-process") };

        load_dotenv_from(&path);

        assert_eq!(
            std::env::var("READMESTUFF_TEST_UNSET_VAR").as_deref(),
            Ok("from-dotenv")
        );
        assert_eq!(
            std::env::var("READMESTUFF_TEST_ALREADY_SET").as_deref(),
            Ok("from-process")
        );

        unsafe { std::env::remove_var("READMESTUFF_TEST_UNSET_VAR") };
        unsafe { std::env::remove_var("READMESTUFF_TEST_ALREADY_SET") };
        std::fs::remove_dir_all(&dir).ok();
    }
}
