use serde::Deserialize;

use std::collections::HashMap;
use std::error;
use std::fmt;

use toml;
use toml::de::Error as TomlError;

use hlua::Lua;

use run_script::ScriptOptions;

/// Defines types for the runner

/// Defines the runtime to use
///
/// - Rune
/// - Shell
#[derive(Deserialize, Debug, PartialEq)]
pub enum Runtime {
    Lua,
    Shell,
}

/// Defines a task to execute and run
/// You are required to have a `main` task
#[derive(Deserialize, Debug)]
pub struct Task {
    /// What runtime to use
    pub runtime: Runtime,
    /// To load script from file?
    pub file: Option<String>,
    /// To have an inline script?
    pub script: Option<String>,
    /// Have a Windows only script?
    pub script_windows: Option<String>,
    /// Have a Linux only script?
    pub script_linux: Option<String>,
    /// Have a MacOs only script?
    pub script_darwin: Option<String>,
}

/// Define the package of what to build
#[derive(Deserialize, Debug)]
pub struct Package {
    /// Name of the package
    pub name: String,
    /// Version of the package
    pub version: String,
    /// Description of the package?
    pub description: Option<String>,
    /// The authors of the package?
    pub authors: Option<Vec<String>>,
}

/// A git submodule to add
#[derive(Deserialize, Debug)]
pub struct GitDependency {
    /// The git url
    pub source: String,
    /// File to save submodule
    pub path: Option<String>,
}

/// A lit package to install
#[derive(Deserialize, Debug)]
pub struct LitDependency {
    /// The author of the package
    pub author: String,
    /// The version of the package
    pub version: Option<String>,
}

/// What this package relies on
#[derive(Deserialize, Debug)]
pub struct Dependencies {
    /// Something that has to exist within PATH
    pub bin: Option<Vec<String>>,
    /// A module to download using git (initializes git)
    pub git: Option<HashMap<String, GitDependency>>,
    /// A module to download using lit
    pub lit: Option<HashMap<String, LitDependency>>,
}

/// A configuration file
#[derive(Deserialize, Debug)]
pub struct Config {
    /// Package metadata
    pub package: Package,
    /// Dependency metadata
    pub dependencies: Option<Dependencies>,
    /// Tasks to run
    pub tasks: HashMap<String, Task>,
}

/// A runner object which is used by most other scripts
pub struct Runner<'a> {
    /// The tasks the runner needs to hold
    pub tasks: &'a HashMap<String, Task>,
    /// Stores the Lua vm if one has been created
    pub lua: Option<Lua<'a>>,
    /// The options for running scripts if the shell is used
    pub options: Option<ScriptOptions>,
}

#[derive(Debug)]
pub enum Error {
    InvalidConfig(TomlError),
    MissingMain,
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidConfig(e) => write!(f, "{}", e),
            Error::MissingMain => write!(f, "Your config file is missing a main method!"),
        }
    }
}

impl From<TomlError> for Error {
    fn from(e: TomlError) -> Self {
        Error::InvalidConfig(e)
    }
}

impl Config {
    /// Create a new configuration
    /// Takes the input and return a result of a config and parse error
    pub fn new(to_parse: &str) -> Result<Config, Error> {
        let config: Config = toml::from_str(to_parse)?;

        if !config.tasks.contains_key("main") {
            return Err(Error::MissingMain);
        }

        Ok(config)
    }

    /// Check if the configuration if that runtime is going to be used
    pub fn uses(&self, runtime: Runtime) -> bool {
        for (_, task) in self.tasks.iter() {
            let matches = match &task.runtime {
                rt => &runtime == rt,
            };

            if matches {
                return true;
            }
        }

        return false;
    }
}

impl<'a> Runner<'a> {
    /// Create a new script runner
    /// Takes the config object and returns a runner
    pub fn new(tasks: &'a HashMap<String, Task>) -> Runner<'a> {
        Runner {
            tasks,
            lua: None,
            options: None,
        }
    }
}
