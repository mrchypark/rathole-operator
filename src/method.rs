use crate::{
	config::initialize_config,
	crd::{Client, Server, ServiceConfig, ServiceType as RH_ServiceType},
	rathole::*,
};

use std::{
	collections::{BTreeMap, HashMap},
	sync::Arc,
};

use kube::{api::ObjectMeta, core::object::HasSpec, Resource};

use k8s_openapi::{
	api::{
		apps::v1::{Deployment, DeploymentSpec, DeploymentStrategy},
		core::v1::{
			Container, ContainerPort, PodSpec, PodTemplateSpec, Secret, SecretVolumeSource, Volume,
			VolumeMount,
		},
	},
	apimachinery::pkg::apis::meta::v1::LabelSelector,
	ByteString,
};

impl ServiceConfig {
	pub fn to_config(&self, token: String) -> ClientServiceConfig {
		return ClientServiceConfig {
			service_type: match self.r#type {
				Some(RH_ServiceType::Tcp) | None => ServiceType::Tcp,
				Some(RH_ServiceType::Udp) => ServiceType::Udp,
			},
			name: self.name.clone(),
			local_addr: format!("{}:{}", self.local_addr.host, self.local_addr.port),
			nodelay: Some(self.nodelay),
			retry_interval: Some(self.retry_interval as u64),
			token: Some(token),
		};
	}
}

impl Config {
	pub fn into_bytes(&self) -> Vec<u8> {
		return toml::to_string(&self).unwrap().into_bytes();
	}
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
	pub fn add_service(&mut self, dat: ClientServiceConfig) {
		self.server.as_mut().unwrap().services.insert(
			dat.name.clone(),
			ServerServiceConfig {
				service_type: ServiceType::Tcp,
				name: dat.name.clone(),
				bind_addr: dat.local_addr,
				token: dat.token,
				nodelay: dat.nodelay,
			},
		);
	}
}

impl Client {
	pub fn to_config(&self) -> Secret {
		let env = initialize_config();
		return Secret {
			metadata: ObjectMeta {
				name: self.spec.config_to.name.clone(),
				namespace: self.spec.config_to.namespace.clone(),
				owner_references: Some(vec![self.controller_owner_ref(&()).unwrap().clone()]),
				..ObjectMeta::default()
			},
			data: Some(BTreeMap::from([(
				String::from("config"),
				ByteString("test".to_string().into_bytes()),
			)])),
			..Default::default()
		};
	}
}

impl Server {
	pub fn to_config_secret(&self) -> Secret {
		let env = initialize_config();
		return Secret {
			metadata: ObjectMeta {
				name: self.metadata.name.clone(),
				owner_references: Some(vec![self.controller_owner_ref(&()).unwrap().clone()]),
				..ObjectMeta::default()
			},
			data: Some(BTreeMap::from([(
				env.rathole_config_name.clone(),
				ByteString(Config::runset(Arc::new(self.clone())).into_bytes()),
			)])),
			..Default::default()
		};
	}
	pub fn to_deployment(&self) -> Deployment {
		let env = initialize_config();
		return Deployment {
			metadata: ObjectMeta {
				name: self.metadata.name.clone(),
				owner_references: Some(vec![self.controller_owner_ref(&()).unwrap().clone()]),
				..ObjectMeta::default()
			},
			spec: Some(DeploymentSpec {
				replicas: Some(1),
				selector: LabelSelector {
					match_labels: Some(BTreeMap::from([
						("name".to_string(), self.metadata.name.clone().unwrap()),
						(
							"app.kubernetes.io/instance".to_string(),
							self.metadata.name.clone().unwrap(),
						),
						(
							"app.kubernetes.io/name".to_string(),
							self.metadata.name.clone().unwrap(),
						),
					])),
					..Default::default()
				},
				template: PodTemplateSpec {
					metadata: Some(ObjectMeta {
						labels: Some(BTreeMap::from([
							("name".to_string(), self.metadata.name.clone().unwrap()),
							(
								"app.kubernetes.io/instance".to_string(),
								self.metadata.name.clone().unwrap(),
							),
							(
								"app.kubernetes.io/name".to_string(),
								self.metadata.name.clone().unwrap(),
							),
						])),
						..Default::default()
					}),
					spec: Some(PodSpec {
						volumes: Some(vec![Volume {
							name: "rathole-config".to_string(),
							secret: Some(SecretVolumeSource {
								optional: Some(false),
								secret_name: self.metadata.name.clone().map(|mut s| {
									s.push_str("-server-config");
									s
								}),
								..Default::default()
							}),
							..Default::default()
						}]),
						containers: vec![Container {
							name: "rathole-server".to_string(),
							image: Some(env.rathole_image.clone()),
							args: Some(vec![
								"--server".to_string(),
								format!(
									"{}/{}",
									env.rathole_config_path.clone(),
									env.rathole_config_name.clone()
								),
							]),
							volume_mounts: Some(vec![VolumeMount {
								read_only: Some(true),
								mount_path: env.rathole_config_path.clone(),
								name: "rathole-config".to_string(),
								..Default::default()
							}]),
							ports: Some(vec![ContainerPort {
								container_port: self.spec().bind_addr.port,
								..Default::default()
							}]),
							..Default::default()
						}],
						..Default::default()
					}),
				},
				strategy: Some(DeploymentStrategy::default()),
				..Default::default()
			}),
			..Default::default()
		};
	}
}
