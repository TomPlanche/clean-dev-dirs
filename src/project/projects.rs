use anyhow::Result;
use colored::Colorize;
use dialoguer::{MultiSelect, theme::ColorfulTheme};
use humansize::{DECIMAL, format_size};

use crate::project::ProjectType;

use super::Project;

pub(crate) struct Projects(Vec<Project>);

impl From<Vec<Project>> for Projects {
    fn from(projects: Vec<Project>) -> Self {
        Self(projects)
    }
}

impl Projects {
    pub(crate) fn get_total_size(&self) -> u64 {
        self.0.iter().map(|p| p.build_arts.size).sum()
    }

    pub(crate) fn interactive_selection(&self) -> Result<Vec<Project>> {
        let items: Vec<String> = self
            .0
            .iter()
            .map(|p| {
                let icon = match p.kind {
                    ProjectType::Rust => "🦀",
                    ProjectType::Node => "📦",
                };
                format!(
                    "{icon} {} ({})",
                    p.root_path.display(),
                    format_size(p.build_arts.size, DECIMAL)
                )
            })
            .collect();

        let defaults = vec![true; self.0.len()];

        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select projects to clean:")
            .items(&items)
            .defaults(&defaults)
            .interact()?;

        Ok(selections.into_iter().map(|i| self.0[i].clone()).collect())
    }

    pub(crate) fn print_summary(&self, total_size: u64) {
        let mut rust_count = 0;
        let mut node_count = 0;
        let mut rust_size = 0u64;
        let mut node_size = 0u64;

        for project in &self.0 {
            match project.kind {
                ProjectType::Rust => {
                    rust_count += 1;
                    rust_size += project.build_arts.size;
                }
                ProjectType::Node => {
                    node_count += 1;
                    node_size += project.build_arts.size;
                }
            }
        }

        if rust_count > 0 {
            println!(
                "  🦀 {} Rust projects ({})",
                rust_count.to_string().bright_white(),
                format_size(rust_size, DECIMAL).bright_white()
            );
        }

        if node_count > 0 {
            println!(
                "  📦 {} Node.js projects ({})",
                node_count.to_string().bright_white(),
                format_size(node_size, DECIMAL).bright_white()
            );
        }

        println!(
            "  💾 Total reclaimable space: {}",
            format_size(total_size, DECIMAL).bright_green().bold()
        );
    }
}
