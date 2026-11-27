use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteVersion {
    pub tool_name: String,
    pub version: String,
    pub lts: Option<String>,
    pub release_date: Option<String>,
    pub download_url: String,
}

pub fn get_node_catalog() -> Vec<RemoteVersion> {
    vec![
        RemoteVersion {
            tool_name: "node".into(),
            version: "20.11.0".into(),
            lts: Some("Iron".into()),
            release_date: Some("2024-01-09".into()),
            download_url: "https://nodejs.org/dist/v20.11.0/node-v20.11.0-win-x64.zip".into(),
        },
        RemoteVersion {
            tool_name: "node".into(),
            version: "20.10.0".into(),
            lts: Some("Iron".into()),
            release_date: Some("2023-11-21".into()),
            download_url: "https://nodejs.org/dist/v20.10.0/node-v20.10.0-win-x64.zip".into(),
        },
        RemoteVersion {
            tool_name: "node".into(),
            version: "18.19.0".into(),
            lts: Some("Hydrogen".into()),
            release_date: Some("2023-11-29".into()),
            download_url: "https://nodejs.org/dist/v18.19.0/node-v18.19.0-win-x64.zip".into(),
        },
        RemoteVersion {
            tool_name: "node".into(),
            version: "18.18.2".into(),
            lts: Some("Hydrogen".into()),
            release_date: Some("2023-10-13".into()),
            download_url: "https://nodejs.org/dist/v18.18.2/node-v18.18.2-win-x64.zip".into(),
        },
        RemoteVersion {
            tool_name: "node".into(),
            version: "21.6.2".into(),
            lts: None,
            release_date: Some("2024-02-06".into()),
            download_url: "https://nodejs.org/dist/v21.6.2/node-v21.6.2-win-x64.zip".into(),
        },
    ]
}
