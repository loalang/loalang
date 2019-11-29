use crate::pkg::api::*;
use graphql_client::Response;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
struct LoapkgConfig {
    auth_token: Option<String>,
    auth_email: Option<String>,
}

impl LoapkgConfig {
    const DEFAULT: LoapkgConfig = LoapkgConfig {
        auth_token: None,
        auth_email: None,
    };
}

pub struct APIClient {
    host: String,
    config_file: PathBuf,
    client: reqwest::Client,
}

impl APIClient {
    pub fn new(host: &str, config_file: &str) -> APIClient {
        APIClient {
            host: host.into(),
            config_file: PathBuf::from(config_file),
            client: reqwest::Client::new(),
        }
    }

    pub fn login(&self, email: &str, password: &str) -> APIResult<()> {
        let query_body = LoginMutation::build_query(login_mutation::Variables {
            email: email.into(),
            password: password.into(),
        });

        let mut response = self
            .client
            .post(self.host.as_str())
            .json(&query_body)
            .send()?;

        let response_body: Response<login_mutation::ResponseData> = response.json()?;

        if response_body.errors.is_some() {
            return Err(APIError::GraphQL(response_body.errors));
        } else if (&response_body.data)
            .as_ref()
            .and_then(|d| d.login.as_ref())
            .is_none()
        {
            return Err(APIError::InvalidCredentials);
        }

        for cookie in response.cookies() {
            if cookie.name() == "LOA_AUTH" {
                let token = cookie.value();

                self.update_config(|config| {
                    config.auth_token = Some(token.into());
                    config.auth_email = Some(response_body.data.unwrap().login.unwrap().email);
                })?;

                return Ok(());
            }
        }

        Err(APIError::InvalidCredentials)
    }

    fn get_config_file(&self) -> APIResult<std::fs::File> {
        Ok(std::fs::OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(&self.config_file)?)
    }

    fn read_config(&self) -> APIResult<LoapkgConfig> {
        Ok(match serde_json::from_reader(self.get_config_file()?) {
            Err(_) => LoapkgConfig::DEFAULT,
            Ok(config) => config,
        })
    }

    fn write_config(&self, config: LoapkgConfig) -> APIResult<()> {
        serde_json::to_writer(self.get_config_file()?, &config)?;
        Ok(())
    }

    fn update_config<F: FnOnce(&mut LoapkgConfig)>(&self, f: F) -> APIResult<()> {
        let mut config = self.read_config()?;
        f(&mut config);
        self.write_config(config)
    }

    pub fn logout(&self) -> APIResult<()> {
        self.update_config(|config| {
            config.auth_token = None;
            config.auth_email = None;
        })
    }

    pub fn auth_email(&self) -> APIResult<Option<String>> {
        Ok(self.read_config()?.auth_email)
    }

    fn auth_token(&self) -> APIResult<Option<String>> {
        Ok(self.read_config()?.auth_token)
    }

    pub fn add_packages(&self, packages: Vec<&str>) -> APIResult<()> {
        for package in packages {
            self.add_package(package)?;
        }
        Ok(())
    }

    fn add_package(&self, package: &str) -> APIResult<()> {
        let mut package_dir = PathBuf::from(".pkg");
        package_dir.extend(package.split("/"));
        std::fs::create_dir_all(&package_dir)?;

        let query_body = GetPackageQuery::build_query(get_package_query::Variables {
            name: package.into(),
        });

        let mut response = self
            .client
            .post(self.host.as_str())
            .json(&query_body)
            .send()?;

        let response_body: Response<get_package_query::ResponseData> = response.json()?;

        if response_body.errors.is_some() {
            return Err(APIError::GraphQL(response_body.errors));
        }

        if let Some(data) = response_body.data {
            if let Some(package) = data.package {
                let name = package.name;
                let version = package.latest_version.version;
                let url = package.latest_version.url;

                println!("Downloading {} version {}", name, version);

                let response = reqwest::get(url.as_str())?;
                let mut archive = tar::Archive::new(response);

                for entry in archive.entries()? {
                    entry?.unpack_in(&package_dir)?;
                }

                return Ok(());
            }
        }

        Err(APIError::PackageNotFound)
    }

    fn add_dir_to_archive(&self, path: &Path, dir: &Path) {}

    fn pack(&self) -> APIResult<Vec<u8>> {
        let mut buf = vec![];
        let mut builder = tar::Builder::new(&mut buf);
        builder.append_dir_all(".", &std::env::current_dir().unwrap())?;
        builder.finish()?;
        drop(builder);
        Ok(buf)
    }

    pub fn publish_package(&self, name: &str, version: &str) -> APIResult<()> {
        let query_body = UploadPackageMutation::build_query(upload_package_mutation::Variables {
            name: name.into(),
            version: version.into(),
            package: Upload,
        });

        let form = reqwest::multipart::Form::new()
            .text("operations", serde_json::to_string(&query_body)?)
            .text("map", r##"{ "0": ["variables.package"] }"##)
            .part(
                "0",
                reqwest::multipart::Part::bytes(self.pack()?)
                    .file_name("package.tar.gz")
                    .mime_str("application/tar+gzip")
                    .unwrap(),
            );

        let mut request = self.client.post(self.host.as_str()).multipart(form);

        if let Some(token) = self.auth_token()? {
            request = request.header("Cookie", format!("LOA_AUTH={}", token));
        }

        let mut response = request.send()?;

        let response_body: Response<upload_package_mutation::ResponseData> = response.json()?;

        match response_body.data.and_then(|data| data.publish_package) {
            None => Err(APIError::GraphQL(response_body.errors)),
            Some(_) => Ok(()),
        }
    }
}
