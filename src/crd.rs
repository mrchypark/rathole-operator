use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Serialize, Deserialize, Default, Debug, PartialEq, Clone, JsonSchema)]
#[kube(
	group = "rathole.mrchypark.github.io",
	version = "v1alpha1",
	kind = "Client",
	plural = "clients",
	derive = "PartialEq",
	shortname = "cl",
	status = "ClientStatus",
	namespaced
)]
pub struct ClientSpec {
	#[serde(rename(deserialize = "serverRef"))]
	pub server_ref: String,
	#[serde(rename(deserialize = "configTo"))]
	pub config_to: ConfigTo,
	/// Timeout for the heartbeat signal in seconds.
	#[serde(default = "heartbeat_timeout_default")]
	#[serde(rename(deserialize = "heartbeatTimeout"))]
	pub heartbeat_timeout: i32,
	#[serde(default = "retry_interval_default")]
	#[serde(rename(deserialize = "retryInterval"))]
	pub retry_interval: i32,
	pub services: Vec<ServiceConfig>,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone, JsonSchema)]
pub struct ConfigTo {
	#[serde(default = "client_config_type_defualt")]
	pub r#type: ClientConfigType,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub namespace: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ClientConfigType {
	#[default]
	Secret,
}

fn client_config_type_defualt() -> ClientConfigType {
	ClientConfigType::Secret
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone, JsonSchema)]
pub struct ServiceConfig {
	pub name: String,
	#[serde(rename(deserialize = "localAddr"))]
	pub local_addr: Uri,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub r#type: Option<ServiceType>,
	#[serde(default = "nodelay_default")]
	pub nodelay: bool,
	#[serde(default = "retry_interval_default")]
	#[serde(rename(deserialize = "retryInterval"))]
	pub retry_interval: i32,
	pub token: Token,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone, JsonSchema)]
pub enum ServiceType {
	#[serde(rename = "tcp")]
	#[default]
	Tcp,
	#[serde(rename = "udp")]
	Udp,
}

fn nodelay_default() -> bool {
	true
}

fn retry_interval_default() -> i32 {
	1
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct ClientStatus {
	pub server_ref: String,
	pub server_config_ok: bool,
	pub server_deploy_ok: bool,
}

#[derive(CustomResource, Serialize, Deserialize, Default, Debug, PartialEq, Clone, JsonSchema)]
#[kube(
	group = "rathole.mrchypark.github.io",
	version = "v1alpha1",
	kind = "Server",
	plural = "servers",
	derive = "PartialEq",
	derive = "Default",
	shortname = "srv",
	status = "ServerStatus",
	namespaced
)]

pub struct ServerSpec {
	#[serde(rename(deserialize = "bindAddr"))]
	pub bind_addr: Uri,
	#[serde(rename(deserialize = "defaultToken"))]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub default_token: Option<String>,
	#[serde(rename(deserialize = "exposeAddr"))]
	pub expose_addr: Uri,
	#[serde(default = "heartbeat_interval_default")]
	#[serde(rename(deserialize = "heartbeatInterval"))]
	pub heartbeat_interval: i32,
	pub transport: Transport,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, JsonSchema)]
pub struct Uri {
	pub host: String,
	pub port: i32,
}

fn heartbeat_interval_default() -> i32 {
	30
}

fn heartbeat_timeout_default() -> i32 {
	30
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, JsonSchema)]
pub struct Transport {
	pub r#type: TransportType,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tcp: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tls: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub websocket: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub noise: Option<NoiseConfig>,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, JsonSchema)]
pub struct NoiseConfig {
	#[serde(default = "noise_pattern_default")]
	pub pattern: String,
	pub noisekey: Token,
}

fn noise_pattern_default() -> String {
	"Noise_NK_25519_ChaChaPoly_BLAKE2s".into()
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, JsonSchema)]
pub struct Token {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub r#type: Option<SecretType>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub key: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(rename(deserialize = "secretRef"))]
	pub secret_ref: Option<SecretRef>,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, JsonSchema)]
pub struct SecretRef {
	pub name: String,
	pub key: String,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TransportType {
	#[default]
	Noise,
	TCP,
	TLS,
	Websocket,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SecretType {
	Direct,
	#[default]
	Reference,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, JsonSchema)]
pub struct ServerStatus {
	pub is_ready: bool,
	pub expose_addr: String,
}
