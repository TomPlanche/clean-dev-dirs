use std::{
    fmt::{Display, Formatter, Result},
    path::PathBuf,
};

#[derive(Clone, Debug, PartialEq)] // TODO: Remove Debug
pub(crate) enum ProjectType {
    Rust,
    Node,
}

#[derive(Clone, Debug)]
pub(crate) struct BuildDirectory {
    pub(crate) path: PathBuf,
    pub(crate) size: u64,
}

#[derive(Clone, Debug)] // TODO: Remove Debug
pub(crate) struct Project {
    /// Type of the project
    pub(crate) kind: ProjectType,

    /// The root directory of the project (where Cargo.toml or package.json is located)
    pub(crate) root_path: PathBuf,

    /// The build directory to be cleaned (`target/` or `node_modules/`)
    pub(crate) build_dir: BuildDirectory,

    /// Name of the project (from Cargo.toml or package.json)
    pub(crate) name: Option<String>,
}

impl Project {
    pub fn new(
        root_path: PathBuf,
        build_dir: BuildDirectory,
        kind: ProjectType,
        name: Option<String>,
    ) -> Self {
        Self {
            root_path,
            build_dir,
            kind,
            name,
        }
    }
}

impl Display for Project {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let icon = match self.kind {
            ProjectType::Rust => "🦀",
            ProjectType::Node => "📦",
        };

        if let Some(name) = &self.name {
            write!(f, "{icon} {name} ({})", self.root_path.display())
        } else {
            write!(f, "{icon} {}", self.root_path.display())
        }
    }
}
