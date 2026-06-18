//! Filters network traffic to Discord-related destinations only.

mod windivert;

use std::collections::HashSet;
use std::net::IpAddr;
use std::path::Path;

pub use windivert::{
    find_windivert_dir, windivert_files_present, windivert_filter, windivert_search_paths,
};

#[derive(Debug, Default)]
pub struct DiscordFilter {
    domains: HashSet<String>,
    cidrs: Vec<String>,
}

impl DiscordFilter {
    pub fn from_lists(domains: &[String], cidrs: &[String]) -> Self {
        Self {
            domains: domains.iter().map(|d| d.trim().to_ascii_lowercase()).collect(),
            cidrs: cidrs.to_vec(),
        }
    }

    pub fn load_from_dir(dir: &Path) -> anyhow::Result<Self> {
        let domains_path = dir.join("discord-domains.txt");
        let domains = read_lines(&domains_path)?;
        let ipset_path = dir.join("discord-ipset.txt");
        let cidrs = if ipset_path.exists() {
            read_lines(&ipset_path)?
        } else {
            Vec::new()
        };
        Ok(Self::from_lists(&domains, &cidrs))
    }

    pub fn matches_domain(&self, host: &str) -> bool {
        let host = host.trim().trim_end_matches('.').to_ascii_lowercase();
        if self.domains.contains(&host) {
            return true;
        }
        self.domains.iter().any(|domain| {
            host == *domain || host.ends_with(&format!(".{domain}"))
        })
    }

    pub fn matches_port(&self, port: u16) -> bool {
        matches!(port, 80 | 443) || (19_294..=19_344).contains(&port) || (50_000..=50_100).contains(&port)
    }

    pub fn matches_ip(&self, _ip: IpAddr) -> bool {
        // TODO: parse CIDR ranges from discord-ipset.txt
        false
    }
}

fn read_lines(path: &Path) -> anyhow::Result<Vec<String>> {
    let content = std::fs::read_to_string(path)?;
    Ok(content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect())
}
