use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub auth_token: Option<String>,
    pub auth_email: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            auth_token: None,
            auth_email: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Lockfile(pub HashMap<String, LockfilePackageRegistration>);

#[derive(Serialize, Deserialize)]
pub struct LockfilePackageRegistration {
    pub version: String,
    pub checksum: String,
    pub url: String,
}

impl Default for Lockfile {
    fn default() -> Self {
        Lockfile(HashMap::new())
    }
}

#[derive(Serialize, Deserialize)]
pub struct Pkgfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<HashMap<String, String>>,
}

impl Default for Pkgfile {
    fn default() -> Self {
        Pkgfile {
            name: None,
            version: None,
            dependencies: None,
        }
    }
}
