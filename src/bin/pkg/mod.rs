mod package_error;
pub use self::package_error::*;

use graphql_client::{GraphQLQuery, Response};
use serde::{Serialize, Serializer};

#[derive(Debug)]
pub struct Upload;

impl Serialize for Upload {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_none()
    }
}

#[derive(GraphQLQuery, Debug)]
#[graphql(
    schema_path = "src/bin/pkg/schema.json",
    query_path = "src/bin/pkg/upload_package_mutation.graphql",
    response_derives = "Debug"
)]
struct UploadPackageMutation;

pub type PackageResult<T> = Result<T, PackageError>;

pub fn add_packages(server_host: &str, names: Vec<&str>) -> PackageResult<()> {
    for name in names {
        add_package(server_host, name)?;
    }
    Ok(())
}

pub fn add_package(_server_host: &str, _name: &str) -> PackageResult<()> {
    Ok(())
}

pub fn publish_package(server_host: &str, name: &str, version: &str) -> PackageResult<()> {
    let query_body = UploadPackageMutation::build_query(upload_package_mutation::Variables {
        name: name.into(),
        version: version.into(),
        package: Upload,
    });

    let client = reqwest::Client::new();
    let form = reqwest::multipart::Form::new()
        .text("operations", serde_json::to_string(&query_body)?)
        .text("map", r##"{ "0": ["variables.package"] }"##)
        .part(
            "0",
            reqwest::multipart::Part::bytes(vec![0u8, 2, 1])
                .file_name("package.tar.gz")
                .mime_str("application/tar+gzip")
                .unwrap(),
        );

    let mut response = client.post(server_host).multipart(form).send()?;
    let response_body: Response<upload_package_mutation::ResponseData> = response.json()?;

    match response_body.data.and_then(|data| data.publish_package) {
        None => Err(PackageError::FailedToUpload(response_body.errors)),
        Some(package) => {
            let name = package.name;
            let version = package.latest_version.version;

            println!("Successfully published {} version {}", name, version);

            Ok(())
        }
    }
}
