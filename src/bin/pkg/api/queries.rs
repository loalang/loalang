use crate::pkg::api::*;
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
    schema_path = "src/bin/pkg/api/schema.json",
    query_path = "src/bin/pkg/api/queries.graphql",
    response_derives = "Debug"
)]
pub struct UploadPackageMutation;

#[derive(GraphQLQuery, Debug)]
#[graphql(
    schema_path = "src/bin/pkg/api/schema.json",
    query_path = "src/bin/pkg/api/queries.graphql",
    response_derives = "Debug"
)]
pub struct LoginMutation;

#[derive(GraphQLQuery, Debug)]
#[graphql(
    schema_path = "src/bin/pkg/api/schema.json",
    query_path = "src/bin/pkg/api/queries.graphql",
    response_derives = "Debug"
)]
pub struct ResolvePackagesQuery;
