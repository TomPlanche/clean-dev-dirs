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
pub(crate) struct BuildArtifacts {
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
    pub(crate) build_arts: BuildArtifacts,

    /// Name of the project (from Cargo.toml or package.json)
    pub(crate) name: Option<String>,
}

impl Project {
    pub fn new(
        kind: ProjectType,
        root_path: PathBuf,
        build_arts: BuildArtifacts,
        name: Option<String>,
    ) -> Self {
        Self {
            kind,
            root_path,
            build_arts,
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
