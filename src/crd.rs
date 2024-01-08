use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
#[kube(
    group = "rathole.mrchypark.github.io",
    version = "v1alpha1",
    kind = "Client",
    plural = "clients",
    derive = "PartialEq",
    namespaced
)]
#[kube(status = "ClientStatus")]
pub struct ClientSpec {
    pub replicas: i32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct ClientStatus {
  pub is_ok: bool,
}

#[derive(CustomResource, Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
#[kube(
    group = "rathole.mrchypark.github.io",
    version = "v1alpha1",
    kind = "Server",
    plural = "servers",
    derive = "PartialEq",
    namespaced
)]
#[kube(status = "ServerStatus")]
pub struct ServerSpec {
    pub replicas: i32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct ServerStatus {
  pub is_ok: bool,
}