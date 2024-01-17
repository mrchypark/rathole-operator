use crate::{
	rathole::{Config, NoiseConfig, ServerConfig, TransportConfig, TransportType},
	Error, Result,
};

use kube::{
	api::{Api, ObjectMeta, Patch, PatchParams},
	client::Client,
	core::object::HasSpec,
	runtime::controller::{Action, Controller},
	Resource, ResourceExt,
};

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

use futures::StreamExt;
use std::{collections::BTreeMap, sync::Arc};
use tokio::time::Duration;

use crate::config::initialize_config;
use crate::crd::{Client as RH_Client, Server as RH_Server};

// Data we want access to in error/reconcile calls
struct Data {
	client: Client,
}

async fn srv_reconcile(obj: Arc<RH_Server>, ctx: Arc<Data>) -> Result<Action> {
	let client = &ctx.client;
	let env = initialize_config();

	println!("server request start: {}", obj.name_any());

	let c = Config {
		client: None,
		server: Some(ServerConfig {
			bind_addr: obj.spec().bind_addr.schema.clone()
				+ "://" + &obj.spec().bind_addr.host
				+ ":" + &obj.spec().bind_addr.port.to_string(),
			default_token: None,
			transport: TransportConfig {
				transport_type: TransportType::Noise,
				noise: Some(NoiseConfig {
					..Default::default()
				}),
				..Default::default()
			},
			heartbeat_interval: obj.spec().heartbeat_interval as u64,
			..Default::default()
		}),
	};

	let t = toml::to_string(&c).unwrap();

	let oref1 = obj.controller_owner_ref(&()).unwrap();
	let oref2 = obj.controller_owner_ref(&()).unwrap();
	// let oref3 = obj.controller_owner_ref(&()).unwrap();

	let mut config = BTreeMap::new();
	config.insert("config.toml".to_string(), ByteString(t.into_bytes()));

	let scr = Secret {
		metadata: ObjectMeta {
			name: obj.metadata.name.clone().map(|mut s| {
				s.push_str("-server-config");
				s
			}),
			owner_references: Some(vec![oref1]),
			..ObjectMeta::default()
		},
		data: Some(config),
		..Default::default()
	};

	let dp_name = obj
		.metadata
		.name
		.clone()
		.map(|mut s| {
			s.push_str("-deployment");
			s
		})
		.unwrap();

	let dp = Deployment {
		metadata: ObjectMeta {
			name: Some(dp_name.clone()),
			owner_references: Some(vec![oref2]),
			..ObjectMeta::default()
		},
		spec: Some(DeploymentSpec {
			replicas: Some(1),
			selector: LabelSelector {
				match_labels: Some(std::collections::BTreeMap::from([
					("name".to_string(), dp_name.clone()),
					("app.kubernetes.io/instance".to_string(), dp_name.clone()),
					("app.kubernetes.io/name".to_string(), dp_name.clone()),
				])),
				..Default::default()
			},
			template: PodTemplateSpec {
				metadata: Some(ObjectMeta {
					labels: Some(std::collections::BTreeMap::from([
						("name".to_string(), dp_name.clone()),
						("app.kubernetes.io/instance".to_string(), dp_name.clone()),
						("app.kubernetes.io/name".to_string(), dp_name.clone()),
					])),
					..Default::default()
				}),
				spec: Some(PodSpec {
					volumes: Some(vec![Volume {
						name: "rathole-config".to_string(),
						secret: Some(SecretVolumeSource {
							optional: Some(false),
							secret_name: obj.metadata.name.clone().map(|mut s| {
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
							env.rathole_config_path.clone(),
						]),
						volume_mounts: Some(vec![VolumeMount {
							read_only: Some(true),
							mount_path: "/tmp".to_string(),
							name: "rathole-config".to_string(),
							..Default::default()
						}]),
						ports: Some(vec![ContainerPort {
							container_port: obj.spec().bind_addr.port,
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

	let dp_api = Api::<Deployment>::namespaced(
		client.clone(),
		obj
			.metadata
			.namespace
			.as_ref()
			.ok_or_else(|| Error::MissingObjectKey(".metadata.namespace"))?,
	);

	let scr_api = Api::<Secret>::namespaced(
		client.clone(),
		obj
			.metadata
			.namespace
			.as_ref()
			.ok_or_else(|| Error::MissingObjectKey(".metadata.namespace"))?,
	);

	let dp_r = dp_api
		.patch(
			dp.metadata
				.name
				.as_ref()
				.ok_or_else(|| Error::MissingObjectKey(".metadata.name"))?,
			&PatchParams::apply("server.rathole.mrchypark.github.io"),
			&Patch::Apply(&dp),
		)
		.await;

	match dp_r {
		Ok(_) => {
			// 성공 로직
		},
		Err(e) => {
			// 에러 처리 로직
			eprintln!("Deployment 생성 실패: {:?}", e);
			// 다른 에러 처리
		},
	}

	scr_api
		.patch(
			scr
				.metadata
				.name
				.as_ref()
				.ok_or_else(|| Error::MissingObjectKey(".metadata.name"))?,
			&PatchParams::apply("server.rathole.mrchypark.github.io"),
			&Patch::Apply(&scr),
		)
		.await
		.map_err(Error::ConfigMapCreationFailed)?;

	println!("server request: {}", obj.name_any());

	Ok(Action::requeue(Duration::from_secs(10)))
}

fn srv_error_policy(obj: Arc<RH_Server>, _err: &Error, _ctx: Arc<Data>) -> Action {
	println!("server request fail: {}", obj.name_any());
	Action::requeue(Duration::from_secs(5))
}

async fn cli_reconcile(obj: Arc<RH_Client>, _ctx: Arc<()>) -> Result<Action> {
	println!("client request: {}", obj.name_any());
	Ok(Action::requeue(Duration::from_secs(10)))
}

fn cli_error_policy(_object: Arc<RH_Client>, _err: &Error, _ctx: Arc<()>) -> Action {
	Action::requeue(Duration::from_secs(5))
}

pub async fn run() {
	let client = Client::try_default()
		.await
		.expect("failed to create kube Client");

	let srv: Api<RH_Server> = Api::all(client.clone());
	let cli: Api<RH_Client> = Api::all(client.clone());
	let dp: Api<Deployment> = Api::all(client.clone());
	let sec: Api<Secret> = Api::all(client.clone());

	let srv_con = Controller::new(srv, Default::default())
		.owns(dp, Default::default())
		.owns(sec, Default::default())
		.shutdown_on_signal()
		.run(srv_reconcile, srv_error_policy, Arc::new(Data { client }))
		.for_each(|_| futures::future::ready(()));

	let cli_con = Controller::new(cli.clone(), Default::default())
		.shutdown_on_signal()
		.run(cli_reconcile, cli_error_policy, Arc::new(()))
		.for_each(|_| futures::future::ready(()));

	let _ = futures::join!(srv_con, cli_con);
}
