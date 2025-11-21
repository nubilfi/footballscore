use parking_lot::{Mutex, MutexGuard};
use serde::Deserialize;
use std::{
    collections::HashMap,
    env::{remove_var, set_var, var_os},
    ffi::{OsStr, OsString},
    ops::Deref,
    path::Path,
    sync::Arc,
};

use crate::{ApiStringType, Error, StringType};

/// Configuration data
#[derive(Default, Debug, Deserialize, PartialEq, Eq)]
pub struct ConfigInner {
    /// api-football.com api key
    pub api_key: Option<ApiStringType>,

    /// api-football.com api endpoint
    #[serde(default = "default_api_endpoint")]
    pub api_endpoint: StringType,

    /// Optional (default is `529 - Barcelona`)
    #[serde(default = "default_club_id")]
    pub club_id: u16,
}

fn default_api_endpoint() -> StringType {
    "v3.football.api-sports.io".into()
}

fn default_club_id() -> u16 {
    529
}

/// Configuration struct
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Config(Arc<ConfigInner>);

impl Config {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Pull in configuration data using `[dotenv](https://crates.io/dotenv)`.
    ///
    /// If a .env file exists in the current directory, pull in any ENV
    /// variables in it.
    ///
    /// Next, if a config file exists in the current directory named config.env,
    /// or if a config file exists at `${HOME}/.config/footballscore/config.env`,
    /// set ENV variables using it.
    ///
    /// Config files should have lines of the following form:
    /// `API_KEY=api_key_value`
    ///
    /// # Example
    ///
    /// ```
    /// # use std::env::set_var;
    /// use footballscore::config::Config;
    /// # use footballscore::config::TestEnvs;
    /// use anyhow::Error;
    ///
    /// # fn main() -> Result<(), Error> {
    /// # let _env = TestEnvs::new(&["API_KEY", "API_ENDPOINT", "CLUB_ID"]);
    /// # set_var("API_KEY", "api_key_value");
    /// # set_var("API_ENDPOINT", "v3.football.api-sports.io");
    /// let config = Config::init_config(None)?;
    /// # drop(_env);
    /// assert_eq!(config.api_key, Some("api_key_value".into()));
    /// assert_eq!(&config.api_endpoint, "v3.football.api-sports.io");
    /// # Ok(())
    /// # }
    /// ```
    /// # Errors
    ///
    /// Will return Error if unable to deserialize env variables
    pub fn init_config(config_path: Option<&Path>) -> Result<Self, Error> {
        let fname = config_path.unwrap_or_else(|| Path::new("config.env"));
        let config_dir = dirs::config_dir().unwrap_or_else(|| "./".into());
        let default_fname = config_dir.join("footballscore").join("config.env");

        let env_file = if fname.exists() {
            fname
        } else {
            &default_fname
        };

        dotenvy::dotenv().ok();

        if env_file.exists() {
            dotenvy::from_path(env_file).ok();
        }

        let conf: ConfigInner = envy::from_env()?;

        Ok(Self(Arc::new(conf)))
    }
}

impl Deref for Config {
    type Target = ConfigInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

static TEST_MUTEX: std::sync::LazyLock<Mutex<()>> = std::sync::LazyLock::new(|| Mutex::new(()));

/// Configuration Test environment
pub struct TestEnvs<'a> {
    _guard: MutexGuard<'a, ()>,
    envs: HashMap<OsString, Option<OsString>>,
}

impl TestEnvs<'_> {
    #[allow(dead_code)]
    pub fn new(keys: &[impl AsRef<OsStr>]) -> Self {
        let guard = TEST_MUTEX.lock();
        let mut envs = HashMap::new();

        for k in keys {
            envs.insert(k.as_ref().to_os_string(), var_os(k.as_ref()));
        }

        Self {
            _guard: guard,
            envs,
        }
    }
}

impl Drop for TestEnvs<'_> {
    fn drop(&mut self) {
        for (key, val) in &self.envs {
            if let Some(val) = val {
                set_var(key, val);
            } else {
                remove_var(key);
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use log::info;
    use std::{
        env::{remove_var, set_var},
        fs::write,
    };
    use tempfile::NamedTempFile;

    use crate::{
        config::{Config, TestEnvs},
        Error,
    };

    #[test]
    fn test_config() -> Result<(), Error> {
        assert_eq!(Config::new(), Config::default());

        let _env = TestEnvs::new(&["API_KEY", "API_ENDPOINT", "CLUB_ID"]);

        set_var("API_KEY", "1e5765fc0c22df4e4ccf20581c2ef3d7");
        set_var("API_ENDPOINT", "test.local");
        set_var("CLUB_ID", "529");

        let conf = Config::init_config(None)?;
        drop(_env);

        info!("{}", conf.api_key.as_ref().unwrap());

        assert_eq!(
            conf.api_key.as_ref().unwrap().as_str(),
            "1e5765fc0c22df4e4ccf20581c2ef3d7"
        );

        #[cfg(feature = "stackstring")]
        assert!(conf.api_key.as_ref().unwrap().is_inline());

        assert_eq!(&conf.api_endpoint, "test.local");
        assert_eq!(conf.club_id, 529);

        Ok(())
    }

    #[test]
    fn test_config_file() -> Result<(), Error> {
        let _env = TestEnvs::new(&["API_KEY", "API_ENDPOINT", "CLUB_ID"]);

        remove_var("API_KEY");
        remove_var("API_ENDPOINT");
        remove_var("CLUB_ID");

        let config_data = include_bytes!("../tests/config/config.env");
        let config_file = NamedTempFile::new()?;
        let config_path = config_file.path();

        write(config_file.path(), config_data)?;

        let conf = Config::init_config(Some(config_path))?;

        assert_eq!(
            conf.api_key.as_ref().unwrap().as_str(),
            "1e5765fc0c22df4e4ccf20581c2ef3d7"
        );

        #[cfg(feature = "stackstring")]
        assert!(conf.api_key.as_ref().unwrap().is_inline());

        assert_eq!(&conf.api_endpoint, "test.local");
        assert_eq!(conf.club_id, 529);

        Ok(())
    }
}
