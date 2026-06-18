use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Profile {
    pub name: ProfileName,
    #[serde(default)]
    pub stages: Vec<Stage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProfileName {
    pub id: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Stage {
    pub protocol: String,
    pub ports: Vec<String>,
    #[serde(default)]
    pub desync: Vec<DesyncMethod>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DesyncMethod {
    Multisplit,
    Fake,
    Fakedsplit,
    Multidisorder,
}

impl Profile {
    pub fn from_toml(input: &str) -> anyhow::Result<Self> {
        Ok(toml::from_str(input)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_profile() {
        let profile = Profile::from_toml(
            r#"
            [name]
            id = "default"
            description = "test"

            [[stages]]
            protocol = "tcp"
            ports = ["443"]
            desync = ["multisplit", "fake"]
            "#,
        )
        .expect("profile should parse");

        assert_eq!(profile.name.id, "default");
        assert_eq!(profile.stages.len(), 1);
        assert_eq!(profile.stages[0].desync.len(), 2);
    }
}
