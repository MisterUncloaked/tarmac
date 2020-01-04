use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};

use crate::glob::Glob;

static CONFIG_FILENAME: &str = "tarmac.toml";

/// Configuration for Tarmac, contained in a tarmac.toml file.
///
/// Tarmac is started from a top-level tarmac.toml file. Config files can
/// include other config files.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Defines project-level fields.
    ///
    /// This field is only expected to be set and is only relevant in the
    /// top-level config that Tarmac runs from.
    pub project: Option<ProjectConfig>,

    /// A list of other Tarmac config files that should be owned by this one.
    #[serde(default)]
    pub includes: Vec<IncludeConfig>,

    /// A list of inputs
    #[serde(default)]
    pub inputs: Vec<InputConfig>,

    /// The path that this config came from. Paths from this config should be
    /// relative to the folder containing this file.
    #[serde(skip)]
    pub file_path: PathBuf,
}

impl Config {
    pub fn read_from_folder_or_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let meta = fs::metadata(path).context(Io { path })?;

        if meta.is_file() {
            Self::read_from_file(path)
        } else {
            Self::read_from_folder(path)
        }
    }

    pub fn folder(&self) -> &Path {
        self.file_path.parent().unwrap()
    }

    fn read_from_file(path: &Path) -> Result<Self, ConfigError> {
        let contents = fs::read(path).context(Io { path })?;

        let mut config: Self = toml::from_slice(&contents).context(Toml { path })?;
        config.file_path = path.to_owned();

        Ok(config)
    }

    fn read_from_folder<P: AsRef<Path>>(folder_path: P) -> Result<Self, ConfigError> {
        let folder_path = folder_path.as_ref();
        let file_path = &folder_path.join(CONFIG_FILENAME);

        Self::read_from_file(file_path)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct ProjectConfig {
    /// A human-readable name for this project.
    pub name: String,

    /// The maximum size that any packed spritesheets should be.
    pub max_spritesheet_size: (usize, usize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct IncludeConfig {
    /// The path to search for other projects in, recursively.
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct InputConfig {
    /// A glob that will match all files that should be considered for this
    /// group of inputs.
    pub glob: Glob,

    /// What kind of extra links Tarmac should generate when these assets are
    /// consumed in a project.
    ///
    /// These links can be used by code located near the affected assets to
    /// import them dynamically as if they were normal Lua modules.
    #[serde(default)]
    pub codegen: CodegenKind,

    /// Whether the assets affected by this config are allowed to be packed into
    /// spritesheets.
    ///
    /// This isn't enabled by default because special considerations need to be
    /// made in order to correctly handle spritesheets. Not all images are able
    /// to be pre-packed into spritesheets, like images used in `Decal`
    /// instances.
    #[serde(default)]
    pub packable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CodegenKind {
    /// Emit no Lua files linking images to their assets.
    ///
    /// This option is useful if another tool is handling the asset mapping, or
    /// assets don't need to be accessed programmatically.
    None,

    /// Emit Lua files that return asset URLs as a string.
    ///
    /// This option is useful for images that will never be packed into a
    /// spritesheet, like `Decal` objects on parts.
    AssetUrl,

    /// Emit Lua files that return a table containing the asset URL, along with
    /// offset and size if the image was packed into a spritesheet.
    ///
    /// The properties in this table are laid out in the same way as the
    /// properties on `ImageLabel` and `ImageButton`:
    ///
    /// * `Image` (string)
    /// * `ImageRectOffset` (Vector2)
    /// * `ImageRectSize` (Vector2)
    UrlAndSlice,
}

impl Default for CodegenKind {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Snafu)]
pub enum ConfigError {
    #[snafu(display("{} in {}", source, path.display()))]
    Toml {
        path: PathBuf,
        source: toml::de::Error,
    },

    #[snafu(display("{} in {}", source, path.display()))]
    Io { path: PathBuf, source: io::Error },
}