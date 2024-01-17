use crate::Result;
use base64::{
	alphabet,
	engine::{self, general_purpose},
	Engine,
};

use kube::core::object::HasSpec;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

use crate::crd::{Server, ServiceConfig};

use std::sync::Arc;

const DEFAULT_CURVE: KeypairType = KeypairType::X25519;

const DEFAULT_NODELAY: bool = true;

const DEFAULT_KEEPALIVE_SECS: u64 = 20;
const DEFAULT_KEEPALIVE_INTERVAL: u64 = 8;

/// Application-layer heartbeat interval in secs
const DEFAULT_HEARTBEAT_INTERVAL_SECS: u64 = 30;
const DEFAULT_HEARTBEAT_TIMEOUT_SECS: u64 = 40;

/// Client
const DEFAULT_CLIENT_RETRY_INTERVAL_SECS: u64 = 1;

impl Config {
	pub fn runset(obj: Arc<Server>) -> Self {
		return Config {
			client: None,
			server: Some(ServerConfig {
				bind_addr: obj.spec().bind_addr.host.clone() + ":" + &obj.spec().bind_addr.port.to_string(),
				default_token: None,
				services: HashMap::from([(
					"dummy".to_string(),
					ServerServiceConfig {
						bind_addr: "0.0.0.0:80".to_string(),
						token: Some("nouse".to_string()),
						..Default::default()
					},
				)]),
				transport: TransportConfig {
					transport_type: TransportType::Noise,
					noise: Some(NoiseConfig {
						pattern: String::from("Noise_NK_25519_ChaChaPoly_BLAKE2s"),
						..Default::default()
					}),
					..Default::default()
				},
				heartbeat_interval: obj.spec().heartbeat_interval as u64,
				..Default::default()
			}),
		};
	}
	pub fn into_bytes(&self) -> Vec<u8> {
		return toml::to_string(&self).unwrap().into_bytes();
	}
	pub fn add_services(&mut self, dat: Vec<ServiceConfig>) {
		for d in &dat {
			self.server.as_mut().unwrap().services.insert(
				d.name.clone(),
				ServerServiceConfig {
					service_type: ServiceType::Tcp,
					name: d.name.clone(),
					bind_addr: format!("{}:{}", d.local_addr.host, d.local_addr.port),
					token: d.token.key.clone(),
					nodelay: Some(d.nodelay),
				},
			);
		}
	}
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(deny_unknown_fields)]
pub struct Config {
	pub server: Option<ServerConfig>,
	pub client: Option<ClientConfig>,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Default)]
pub enum TransportType {
	#[default]
	#[serde(rename = "tcp")]
	Tcp,
	#[serde(rename = "tls")]
	Tls,
	#[serde(rename = "noise")]
	Noise,
	#[serde(rename = "websocket")]
	Websocket,
}

/// Per service config
/// All Option are optional in configuration but must be Some value in runtime
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct ClientServiceConfig {
	#[serde(rename = "type", default = "default_service_type")]
	pub service_type: ServiceType,
	#[serde(skip)]
	pub name: String,
	pub local_addr: String,
	pub token: Option<String>,
	pub nodelay: Option<bool>,
	pub retry_interval: Option<u64>,
}

impl ClientServiceConfig {
	pub fn with_name(name: &str) -> ClientServiceConfig {
		ClientServiceConfig {
			name: name.to_string(),
			..Default::default()
		}
	}
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum ServiceType {
	#[serde(rename = "tcp")]
	#[default]
	Tcp,
	#[serde(rename = "udp")]
	Udp,
}

fn default_service_type() -> ServiceType {
	Default::default()
}

/// Per service config
/// All Option are optional in configuration but must be Some value in runtime
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct ServerServiceConfig {
	#[serde(rename = "type", default = "default_service_type")]
	pub service_type: ServiceType,
	#[serde(skip)]
	pub name: String,
	pub bind_addr: String,
	pub token: Option<String>,
	pub nodelay: Option<bool>,
}

impl ServerServiceConfig {
	pub fn with_name(name: &str) -> ServerServiceConfig {
		ServerServiceConfig {
			name: name.to_string(),
			..Default::default()
		}
	}
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TlsConfig {
	pub hostname: Option<String>,
	pub trusted_root: Option<String>,
	pub pkcs12: Option<String>,
	pub pkcs12_password: Option<String>,
}

fn default_noise_pattern() -> String {
	String::from("Noise_NK_25519_ChaChaPoly_BLAKE2s")
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NoiseConfig {
	#[serde(default = "default_noise_pattern")]
	pub pattern: String,
	pub local_private_key: Option<String>,
	pub remote_public_key: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WebsocketConfig {
	pub tls: bool,
}

fn default_nodelay() -> bool {
	DEFAULT_NODELAY
}

fn default_keepalive_secs() -> u64 {
	DEFAULT_KEEPALIVE_SECS
}

fn default_keepalive_interval() -> u64 {
	DEFAULT_KEEPALIVE_INTERVAL
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TcpConfig {
	#[serde(default = "default_nodelay")]
	pub nodelay: bool,
	#[serde(default = "default_keepalive_secs")]
	pub keepalive_secs: u64,
	#[serde(default = "default_keepalive_interval")]
	pub keepalive_interval: u64,
	pub proxy: Option<Url>,
}

impl Default for TcpConfig {
	fn default() -> Self {
		Self {
			nodelay: default_nodelay(),
			keepalive_secs: default_keepalive_secs(),
			keepalive_interval: default_keepalive_interval(),
			proxy: None,
		}
	}
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct TransportConfig {
	#[serde(rename = "type")]
	pub transport_type: TransportType,
	pub tcp: Option<TcpConfig>,
	pub tls: Option<TlsConfig>,
	#[serde(default)]
	pub noise: Option<NoiseConfig>,
	pub websocket: Option<WebsocketConfig>,
}

fn default_heartbeat_timeout() -> u64 {
	DEFAULT_HEARTBEAT_TIMEOUT_SECS
}

fn default_client_retry_interval() -> u64 {
	DEFAULT_CLIENT_RETRY_INTERVAL_SECS
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq, Clone)]
#[serde(deny_unknown_fields)]
pub struct ClientConfig {
	pub remote_addr: String,
	pub default_token: Option<String>,
	pub services: HashMap<String, ClientServiceConfig>,
	#[serde(default)]
	pub transport: TransportConfig,
	#[serde(default = "default_heartbeat_timeout")]
	pub heartbeat_timeout: u64,
	#[serde(default = "default_client_retry_interval")]
	pub retry_interval: u64,
}

fn default_heartbeat_interval() -> u64 {
	DEFAULT_HEARTBEAT_INTERVAL_SECS
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq, Clone)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
	pub bind_addr: String,
	pub default_token: Option<String>,
	#[serde(default = "dummy_service")]
	pub services: HashMap<String, ServerServiceConfig>,
	#[serde(default)]
	pub transport: TransportConfig,
	#[serde(default = "default_heartbeat_interval")]
	pub heartbeat_interval: u64,
}

fn dummy_service() -> HashMap<String, ServerServiceConfig> {
	return HashMap::from([(
		"dummy".to_string(),
		ServerServiceConfig {
			..Default::default()
		},
	)]);
}

#[derive(Clone, Debug, Copy)]
pub enum KeypairType {
	X25519,
	X448,
}

fn get_str_from_keypair_type(curve: KeypairType) -> &'static str {
	match curve {
		KeypairType::X25519 => "25519",
		KeypairType::X448 => "448",
	}
}

pub fn genkey(curve: Option<KeypairType>) -> Result<(), snowstorm::snow::Error> {
	let curve = curve.unwrap_or(DEFAULT_CURVE);
	let builder = snowstorm::Builder::new(
		format!(
			"Noise_KK_{}_ChaChaPoly_BLAKE2s",
			get_str_from_keypair_type(curve)
		)
		.parse()?,
	);
	let keypair = builder.generate_keypair()?;

	const ENGINE: engine::GeneralPurpose =
		engine::GeneralPurpose::new(&alphabet::STANDARD, general_purpose::NO_PAD);

	println!("Private Key:\n{}\n", ENGINE.encode(keypair.private));
	println!("Public Key:\n{}", ENGINE.encode(keypair.public));

	Ok(())
}
