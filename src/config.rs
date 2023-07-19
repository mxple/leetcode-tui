use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tokio::{fs::File, io::AsyncReadExt};

use serde::{self, Deserialize, Serialize};
use toml;
use xdg::{self, BaseDirectories};

use crate::errors::AppResult;

pub async fn write_file(path: PathBuf, contents: &str) -> AppResult<()> {
    let mut file = File::create(path).await?;
    file.write(contents.as_bytes()).await?;
    Ok(())
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub db: Db,
    pub leetcode: Leetcode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            db: Db::default(),
            leetcode: Leetcode::default(),
        }
    }
}

impl Config {
    pub fn get_base_directory() -> AppResult<BaseDirectories> {
        Ok(xdg::BaseDirectories::with_prefix("leetcode_tui")?)
    }

    pub fn get_base_config() -> AppResult<PathBuf> {
        let config_path = Self::get_base_directory()?.place_config_file("config.toml")?;
        Ok(config_path)
    }

    pub async fn read_config(path: PathBuf) -> AppResult<Self> {
        let mut f = File::open(path).await?;
        let mut contents = String::new();
        f.read_to_string(&mut contents).await?;
        Ok(toml::from_str(contents.as_str())?)
    }

    pub async fn write_config(&self, path: PathBuf) -> AppResult<()> {
        write_file(path, toml::to_string(self)?.as_str()).await?;
        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
pub struct Db {
    pub url: String,
}

impl Db {
    pub fn get_base_sqlite_data_path() -> AppResult<PathBuf> {
        let base_dirs = Config::get_base_directory()?;
        let data_file_path = base_dirs.place_data_file("data.sqlite")?;
        Ok(data_file_path)
    }

    pub async fn touch_default_db() -> AppResult<()> {
        let path = Self::get_base_sqlite_data_path()?;
        write_file(path, "").await?;
        Ok(())
    }
}

impl Default for Db {
    fn default() -> Self {
        Self {
            url: format!(
                "sqlite://{}",
                Self::get_base_sqlite_data_path()
                    .expect("cannot place sqlite data file")
                    .display()
            ),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Leetcode {
    #[serde(rename = "LEETCODE_SESSION")]
    pub leetcode_session: String,
    pub csrftoken: String,
}

impl Default for Leetcode {
    fn default() -> Self {
        Self {
            leetcode_session: "".to_owned(),
            csrftoken: "".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let sample_config = [
            "[db]",
            "url = 'sqlite://leetcode.sqlite'",
            "[leetcode]",
            "csrftoken = 'ctoken'",
            "LEETCODE_SESSION = 'lsession'",
        ]
        .join("\n");

        let config: Config = toml::from_str(sample_config.as_str()).unwrap();
        assert_eq!(config.leetcode.csrftoken, "ctoken".to_string());
        assert_eq!(config.leetcode.leetcode_session, "lsession".to_string());

        assert_eq!(config.db.url, "sqlite://leetcode.sqlite".to_string());
    }
}
