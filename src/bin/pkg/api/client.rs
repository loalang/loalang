use crate::pkg::api::*;
use colored::Colorize;
use crypto::digest::Digest;
use graphql_client::Response;
use loa::HashMap;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

#[derive(Serialize, Deserialize)]
struct LoapkgLockfile(pub HashMap<String, LoapkgLockfilePackageRegistration>);

#[derive(Serialize, Deserialize)]
struct LoapkgLockfilePackageRegistration {
    version: String,
    checksum: String,
    url: String,
}

#[derive(Serialize, Deserialize)]
struct LoapkgPkgfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dependencies: Option<HashMap<String, String>>,
}

impl LoapkgPkgfile {
    const DEFAULT: LoapkgPkgfile = LoapkgPkgfile {
        name: None,
        version: None,
        dependencies: None,
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
                    Ok(())
                })?;

                return Ok(());
            }
        }

        Err(APIError::InvalidCredentials)
    }

    fn get_pkgfile(&self) -> APIResult<std::fs::File> {
        Ok(std::fs::OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open("pkg.yml")?)
    }

    fn read_pkgfile(&self) -> APIResult<LoapkgPkgfile> {
        Ok(match serde_yaml::from_reader(self.get_pkgfile()?) {
            Err(_) => LoapkgPkgfile::DEFAULT,
            Ok(config) => config,
        })
    }

    fn write_pkgfile(&self, config: LoapkgPkgfile) -> APIResult<()> {
        serde_yaml::to_writer(self.get_pkgfile()?, &config)?;
        Ok(())
    }

    fn update_pkgfile<F: FnOnce(&mut LoapkgPkgfile) -> APIResult<()>>(
        &self,
        f: F,
    ) -> APIResult<()> {
        let mut config = self.read_pkgfile()?;
        f(&mut config)?;
        self.write_pkgfile(config)
    }

    fn get_lockfile(&self) -> APIResult<std::fs::File> {
        Ok(std::fs::OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(".pkg.lock")?)
    }

    fn read_lockfile(&self) -> APIResult<LoapkgLockfile> {
        Ok(serde_yaml::from_reader(self.get_lockfile()?).unwrap_or(LoapkgLockfile(HashMap::new())))
    }

    fn write_lockfile(&self, lock: LoapkgLockfile) -> APIResult<()> {
        serde_yaml::to_writer(self.get_lockfile()?, &lock)?;
        Ok(())
    }

    fn update_lockfile<F: FnOnce(&mut LoapkgLockfile) -> APIResult<()>>(
        &self,
        f: F,
    ) -> APIResult<()> {
        let mut lock = self.read_lockfile()?;
        f(&mut lock)?;
        self.write_lockfile(lock)
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

    fn update_config<F: FnOnce(&mut LoapkgConfig) -> APIResult<()>>(&self, f: F) -> APIResult<()> {
        let mut config = self.read_config()?;
        f(&mut config)?;
        self.write_config(config)
    }

    pub fn logout(&self) -> APIResult<()> {
        self.update_config(|config| {
            config.auth_token = None;
            config.auth_email = None;
            Ok(())
        })
    }

    pub fn auth_email(&self) -> APIResult<Option<String>> {
        Ok(self.read_config()?.auth_email)
    }

    fn auth_token(&self) -> APIResult<Option<String>> {
        Ok(self.read_config()?.auth_token)
    }

    pub fn add_packages(&self, packages: Vec<&str>) -> APIResult<()> {
        self.update_lockfile(|lock| {
            self.update_pkgfile(|pkgfile| {
                for package in packages {
                    self.add_package(package, pkgfile, lock)?;
                }
                Ok(())
            })
        })
    }

    fn add_package(
        &self,
        package: &str,
        pkgfile: &mut LoapkgPkgfile,
        lock: &mut LoapkgLockfile,
    ) -> APIResult<()> {
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
                let checksum = package.latest_version.checksum;

                let response = reqwest::get(url.as_str())?;
                let mut archive = tar::Archive::new(response);

                for entry in archive.entries()? {
                    entry?.unpack_in(&package_dir)?;
                }

                println!(
                    "{} {} {}",
                    "Installed".bright_black(),
                    name.green(),
                    format!(" {} ", version).bold().black().on_bright_yellow()
                );

                if let None = pkgfile.dependencies {
                    pkgfile.dependencies = Some(HashMap::new());
                }
                pkgfile
                    .dependencies
                    .as_mut()
                    .unwrap()
                    .insert(name.clone(), version.clone());
                lock.0.insert(
                    name,
                    LoapkgLockfilePackageRegistration {
                        checksum,
                        version,
                        url,
                    },
                );

                return Ok(());
            }
        }

        Err(APIError::PackageNotFound)
    }

    fn walk() -> ignore::Walk {
        let mut builder = ignore::WalkBuilder::new("./");

        builder.add_ignore(".pkgignore");
        builder.add_custom_ignore_filename(".pkg/");
        builder.build()
    }

    fn pack(&self) -> APIResult<Vec<u8>> {
        let mut buf = vec![];
        let mut builder = tar::Builder::new(&mut buf);
        for entry in Self::walk() {
            let entry = entry?.into_path();
            if entry.is_file() {
                println!(
                    "{} {}",
                    "Packing".bright_black(),
                    entry.to_str().unwrap_or("<<unknown>>").green()
                );
                builder.append_path(entry)?;
            }
        }
        builder.finish()?;
        drop(builder);
        Ok(buf)
    }

    pub fn publish_package(&self, name: &str, version: &str) -> APIResult<()> {
        let package = self.pack()?;

        let mut checksum = crypto::sha1::Sha1::new();
        checksum.input(package.as_slice());

        let query_body = UploadPackageMutation::build_query(upload_package_mutation::Variables {
            name: name.into(),
            version: version.into(),
            package: Upload,
            checksum: checksum.result_str(),
        });

        let form = reqwest::multipart::Form::new()
            .text("operations", serde_json::to_string(&query_body)?)
            .text("map", r##"{ "0": ["variables.package"] }"##)
            .part(
                "0",
                reqwest::multipart::Part::bytes(package)
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
