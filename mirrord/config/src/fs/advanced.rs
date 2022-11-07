use mirrord_config_derive::MirrordConfig;
use schemars::JsonSchema;
use serde::Deserialize;

use super::FsModeConfig;
use crate::{
    config::{from_env::FromEnv, source::MirrordConfigSource, ConfigError},
    util::{MirrordToggleableConfig, VecOrSingle},
};

// TODO(alex): We could turn this derive macro (`MirrordConfig`) into an attribute version, which
// would allow us to "capture" the `derive` statement, making it possible to implement the same for
// whatever is generated by `map_to`.
/// Advanced user configuration for file operations.
///
/// Allows the user to specify:
///
/// - `MIRRORD_FILE_OPS` and `MIRRORD_FILE_RO_OPS`;
/// - `MIRRORD_FILE_FILTER_INCLUDE` and `MIRRORD_FILE_FILTER_EXCLUDE`;
///
/// ## Examples
///
/// - Read-only excluding `.foo` files:
///
/// ```yaml
/// # mirrord-config.yaml
///
/// [fs]
/// mode = read
/// exclude = "^.*\.foo$"
/// ```
///
/// - Read-write including only `.baz` files:
///
/// ```yaml
/// # mirrord-config.yaml
///
/// [fs]
/// mode = write
/// include = "^.*\.baz$"
/// ```
#[derive(MirrordConfig, Default, Deserialize, PartialEq, Eq, Clone, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
#[config(map_to = FsConfig)]
pub struct AdvancedFsUserConfig {
    /// File operations mode, defaults to read-only, see [`FsModeConfig`].
    #[serde(default)]
    #[config(nested)]
    pub mode: FsModeConfig,

    /// Allows the user to specify regexes that are used to match against files when mirrord file
    /// operations are enabled.
    ///
    /// The regexes specified here will make mirrord operate only on files that match it, otherwise
    /// the file will be accessed locally (bypassing mirrord).
    #[config(env = "MIRRORD_FILE_FILTER_INCLUDE")]
    pub include: Option<VecOrSingle<String>>,

    /// Allows the user to specify regexes that are used to match against files when mirrord file
    /// operations are enabled.
    ///
    /// The opposite of `include`, files that match the regexes specified here will bypass mirrord
    /// and are accessed locally.
    #[config(env = "MIRRORD_FILE_FILTER_EXCLUDE")]
    pub exclude: Option<VecOrSingle<String>>,
}

impl MirrordToggleableConfig for AdvancedFsUserConfig {
    fn disabled_config() -> Result<Self::Generated, ConfigError> {
        let mode = FsModeConfig::disabled_config()?;
        let include = FromEnv::new("MIRRORD_FILE_FILTER_INCLUDE").source_value();
        let exclude = FromEnv::new("MIRRORD_FILE_FILTER_EXCLUDE").source_value();

        Ok(Self::Generated {
            mode,
            include,
            exclude,
        })
    }
}

impl FsConfig {
    pub fn is_read(&self) -> bool {
        self.mode.is_read()
    }

    pub fn is_write(&self) -> bool {
        self.mode.is_write()
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::{config::MirrordConfig, util::testing::with_env_vars};

    #[rstest]
    fn test_advanced_fs_config_default() {
        let expect = FsConfig {
            mode: FsModeConfig::Read,
            ..Default::default()
        };

        with_env_vars(
            vec![
                ("MIRRORD_FILE_OPS", None),
                ("MIRRORD_FILE_RO_OPS", None),
                ("MIRRORD_FILE_FILTER_INCLUDE", None),
                ("MIRRORD_FILE_FILTER_EXCLUDE", None),
            ],
            || {
                let fs_config = AdvancedFsUserConfig::default().generate_config().unwrap();

                assert_eq!(fs_config, expect);
            },
        );
    }

    #[rstest]
    fn test_advanced_fs_config_file_filter_include() {
        let expect = FsConfig {
            mode: FsModeConfig::Read,
            include: Some(VecOrSingle::Single(".*".to_string())),
            ..Default::default()
        };

        with_env_vars(
            vec![
                ("MIRRORD_FILE_OPS", None),
                ("MIRRORD_FILE_RO_OPS", None),
                ("MIRRORD_FILE_FILTER_INCLUDE", None),
                ("MIRRORD_FILE_FILTER_EXCLUDE", None),
            ],
            || {
                let fs_config = AdvancedFsUserConfig {
                    include: Some(VecOrSingle::Single(".*".to_string())),
                    ..Default::default()
                }
                .generate_config()
                .unwrap();

                assert_eq!(fs_config, expect);
            },
        );
    }
}