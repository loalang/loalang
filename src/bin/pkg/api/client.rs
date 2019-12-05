use crate::pkg::api::*;
use colored::Colorize;
use crypto::digest::Digest;
use graphql_client::Response;
use loa::HashMap;
use std::io::Error;
use std::path::PathBuf;

pub struct APIClient {
    host: String,
    client: reqwest::Client,

    config: ManifestFile<config::Config>,
    lockfile: ManifestFile<config::Lockfile>,
    pkgfile: ManifestFile<config::Pkgfile>,
}

impl APIClient {
    pub fn new(host: &str, config_file: &str) -> APIClient {
        APIClient {
            host: host.into(),
            client: reqwest::Client::new(),

            config: ManifestFile::new(config_file),
            lockfile: ManifestFile::new(".pkg.lock"),
            pkgfile: ManifestFile::new("pkg.yml"),
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

                self.config.update(|config| {
                    config.auth_token = Some(token.into());
                    config.auth_email = Some(response_body.data.unwrap().login.unwrap().email);
                    Ok(())
                })?;

                return Ok(());
            }
        }

        Err(APIError::InvalidCredentials)
    }

    pub fn logout(&self) -> APIResult<()> {
        self.config.update(|config| {
            config.auth_token = None;
            config.auth_email = None;
            Ok(())
        })
    }

    pub fn auth_email(&self) -> APIResult<Option<String>> {
        Ok(self.config.load()?.auth_email)
    }

    fn auth_token(&self) -> APIResult<Option<String>> {
        Ok(self.config.load()?.auth_token)
    }

    pub fn add_packages(&self, packages: HashMap<&str, Option<&str>>) -> APIResult<()> {
        self.lockfile.update(|lock| {
            self.pkgfile.update(|pkgfile| {
                self.add_packages_impl(packages, pkgfile, lock)?;
                Ok(())
            })?;

            self.install_from_lockfile(lock)
        })
    }

    pub fn remove_packages(&self, packages: Vec<&str>) -> APIResult<()> {
        self.pkgfile.update(|pkgfile| {
            if let Some(ref mut d) = pkgfile.dependencies {
                for p in packages {
                    d.remove(p);
                }
            }
            Ok(())
        })?;
        self.get_from_pkgfile()
    }

    fn install_from_lockfile(&self, lock: &mut config::Lockfile) -> APIResult<()> {
        let pkg_dir = PathBuf::from(".pkg");
        if pkg_dir.exists() {
            std::fs::remove_dir_all(&pkg_dir)?;
        }

        for (
            name,
            config::LockfilePackageRegistration {
                version,
                checksum,
                url,
            },
        ) in lock.0.iter()
        {
            let mut response = reqwest::get(url.as_str())?;
            let mut buf = vec![];
            response.copy_to(&mut buf)?;

            let mut checksum_verify = crypto::sha1::Sha1::new();
            checksum_verify.input(buf.as_slice());

            if checksum_verify.result_str() != *checksum {
                return Err(APIError::ChecksumMismatch);
            }

            let mut archive = tar::Archive::new(buf.as_slice());

            let mut package_dir = pkg_dir.clone();
            package_dir.extend(name.split("/"));
            std::fs::create_dir_all(&package_dir)?;
            for entry in archive.entries()? {
                let mut entry = entry?;
                if !entry.unpack_in(&package_dir)? {
                    println!(
                        "{} {}{}{}",
                        "Failed to unpack".bright_black(),
                        package_dir.to_str().unwrap().green(),
                        "/".green(),
                        entry.path()?.to_str().unwrap().green()
                    );
                }
            }

            println!(
                "{} {} {}",
                "Installed".bright_black(),
                name.green(),
                format!(" {} ", version).bold().black().on_bright_yellow()
            );
        }
        Ok(())
    }

    fn add_packages_impl(
        &self,
        packages: HashMap<&str, Option<&str>>,
        pkgfile: &mut config::Pkgfile,
        lock: &mut config::Lockfile,
    ) -> APIResult<()> {
        let no_deps = HashMap::new();
        let requested_packages = pkgfile
            .dependencies
            .as_ref()
            .unwrap_or(&no_deps)
            .iter()
            .map(|(name, version)| resolve_packages_query::RequestedPackage {
                name: name.clone(),
                version: Some(version.clone()),
            })
            .chain(packages.iter().map(|(name, version)| {
                resolve_packages_query::RequestedPackage {
                    name: (*name).into(),
                    version: version.map(String::from),
                }
            }))
            .collect::<Vec<_>>();

        let query_body = ResolvePackagesQuery::build_query(resolve_packages_query::Variables {
            packages: requested_packages,
        });

        let mut response = self
            .client
            .post(self.host.as_str())
            .json(&query_body)
            .send()?;

        let response_body: Response<resolve_packages_query::ResponseData> = response.json()?;

        if response_body.errors.is_some() {
            return Err(APIError::GraphQL(response_body.errors));
        }

        if let Some(data) = response_body.data {
            lock.0.clear();

            if let None = pkgfile.dependencies {
                pkgfile.dependencies = Some(HashMap::new());
            }
            for package in packages.keys() {
                match data
                    .resolve_packages
                    .iter()
                    .filter(|r| r.package.name == *package)
                    .next()
                {
                    None => return Err(APIError::PackageNotFound),
                    Some(r) => {
                        pkgfile
                            .dependencies
                            .as_mut()
                            .unwrap()
                            .insert(r.package.name.clone(), r.version.clone());
                    }
                }
            }

            for release in data.resolve_packages {
                let name = release.package.name;
                let version = release.version;
                let url = release.url;
                let checksum = release.checksum;

                lock.0.insert(
                    name,
                    config::LockfilePackageRegistration {
                        checksum,
                        version,
                        url,
                    },
                );
            }
            return Ok(());
        }

        Err(APIError::PackageNotFound)
    }

    fn walk() -> ignore::Walk {
        let mut builder = ignore::WalkBuilder::new("./");

        builder.add_ignore(".pkgignore");
        builder.add_custom_ignore_filename(".pkg/");
        builder.build()
    }

    fn pack(&self, checksum: &mut ChecksumWriter) -> APIResult<Vec<u8>> {
        let mut buf = vec![];
        let mut builder = tar::Builder::<DualWriter<_, _>>::new((&mut buf, checksum).into());
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
        let mut checksum = ChecksumWriter::new();
        let package = self.pack(&mut checksum)?;

        let query_body = UploadPackageMutation::build_query(upload_package_mutation::Variables {
            name: name.into(),
            version: version.into(),
            package: Upload,
            checksum: checksum.result(),
            dependencies: self
                .pkgfile
                .load()
                .ok()
                .and_then(|p| p.dependencies)
                .map(|deps| {
                    deps.into_iter()
                        .map(
                            |(package, version)| upload_package_mutation::PublicationDependency {
                                package,
                                version,
                                development: None,
                            },
                        )
                        .collect::<Vec<_>>()
                })
                .unwrap_or(vec![]),
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

    pub fn get_from_pkgfile(&self) -> APIResult<()> {
        self.add_packages(HashMap::new())
    }

    pub fn get_from_lockfile(&self) -> APIResult<()> {
        self.lockfile
            .update(|lock| self.install_from_lockfile(lock))
    }
}

struct ChecksumWriter {
    inner: crypto::sha1::Sha1,
}

impl ChecksumWriter {
    pub fn new() -> ChecksumWriter {
        ChecksumWriter {
            inner: crypto::sha1::Sha1::new(),
        }
    }

    pub fn result(&mut self) -> String {
        self.inner.result_str()
    }
}

impl std::io::Write for ChecksumWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        let len = buf.len();
        self.inner.input(buf);
        Ok(len)
    }

    fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

struct DualWriter<A, B> {
    a: A,
    b: B,
}

impl<A, B> From<(A, B)> for DualWriter<A, B> {
    fn from((a, b): (A, B)) -> Self {
        DualWriter { a, b }
    }
}

impl<A: std::io::Write, B: std::io::Write> std::io::Write for DualWriter<A, B> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.a.write(buf).and(self.b.write(buf))
    }

    fn flush(&mut self) -> Result<(), Error> {
        self.a.flush().and(self.b.flush())
    }
}
