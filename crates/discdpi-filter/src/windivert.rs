use std::path::{Path, PathBuf};

/// WinDivert filter matching outbound Discord gateway and voice traffic.
pub fn windivert_filter() -> String {
    [
        "(outbound and tcp and (tcp.DstPort == 80 or tcp.DstPort == 443))",
        "(outbound and udp and (udp.DstPort == 443",
        "(udp.DstPort >= 19294 and udp.DstPort <= 19344)",
        "(udp.DstPort >= 50000 and udp.DstPort <= 50100)))",
    ]
    .join(" or ")
}

/// Candidate directories that may contain WinDivert runtime files.
pub fn windivert_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join("vendor/windivert/x64"));
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            paths.push(parent.join("vendor/windivert/x64"));
            paths.push(parent.join("windivert"));
            paths.push(parent.to_path_buf());
        }
    }

    paths.push(PathBuf::from("vendor/windivert/x64"));
    paths
}

pub fn find_windivert_dir() -> Option<PathBuf> {
    windivert_search_paths()
        .into_iter()
        .find(|dir| windivert_files_present(dir))
}

pub fn windivert_files_present(dir: &Path) -> bool {
    dir.join("WinDivert.dll").is_file() && dir.join("WinDivert64.sys").is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_includes_discord_ports() {
        let filter = windivert_filter();
        assert!(filter.contains("tcp.DstPort == 443"));
        assert!(filter.contains("19294"));
        assert!(filter.contains("50000"));
    }
}
