use crate::{rathole::Config, Error, Result};

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
	let c = Config::runset(obj.clone());

	let oref = obj.controller_owner_ref(&()).unwrap();

	let scr = Secret {
		metadata: ObjectMeta {
			name: obj.metadata.name.clone().map(|mut s| {
				s.push_str("-server-config");
				s
			}),
			owner_references: Some(vec![oref.clone()]),
			..ObjectMeta::default()
		},
		data: Some(BTreeMap::from([(
			env.rathole_config_name.clone(),
			ByteString(c.into_bytes()),
		)])),
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
			owner_references: Some(vec![oref.clone()]),
			..ObjectMeta::default()
		},
		spec: Some(DeploymentSpec {
			replicas: Some(1),
			selector: LabelSelector {
				match_labels: Some(BTreeMap::from([
					("name".to_string(), dp_name.clone()),
					("app.kubernetes.io/instance".to_string(), dp_name.clone()),
					("app.kubernetes.io/name".to_string(), dp_name.clone()),
				])),
				..Default::default()
			},
			template: PodTemplateSpec {
				metadata: Some(ObjectMeta {
					labels: Some(BTreeMap::from([
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

	dp_api
		.patch(
			dp.metadata
				.name
				.as_ref()
				.ok_or_else(|| Error::MissingObjectKey(".metadata.name"))?,
			&PatchParams::apply("server.rathole.mrchypark.github.io"),
			&Patch::Apply(&dp),
		)
		.await
		.map_err(Error::ConfigMapCreationFailed)?;

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

	Ok(Action::requeue(Duration::from_secs(10)))
}

fn srv_error_policy(obj: Arc<RH_Server>, err: &Error, _ctx: Arc<Data>) -> Action {
	println!("server request fail: {}", obj.name_any());
	println!("{}", err);
	Action::requeue(Duration::from_secs(5))
}

async fn cli_reconcile(obj: Arc<RH_Client>, ctx: Arc<Data>) -> Result<Action> {
	let client = &ctx.client;
	let oref = obj.controller_owner_ref(&()).unwrap();

	let scr = Secret {
		metadata: ObjectMeta {
			name: obj.spec.config_to.name.clone(),
			namespace: obj.spec.config_to.namespace.clone(),
			owner_references: Some(vec![oref.clone()]),
			..ObjectMeta::default()
		},
		data: Some(BTreeMap::from([(
			String::from("config"),
			ByteString("test".to_string().into_bytes()),
		)])),
		..Default::default()
	};

	let scr_api = Api::<Secret>::namespaced(
		client.clone(),
		obj
			.metadata
			.namespace
			.as_ref()
			.ok_or_else(|| Error::MissingObjectKey(".metadata.namespace"))?,
	);

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

	Ok(Action::requeue(Duration::from_secs(10)))
}

fn cli_error_policy(obj: Arc<RH_Client>, err: &Error, _ctx: Arc<Data>) -> Action {
	println!("client request fail: {}", obj.name_any());
	println!("{}", err);
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
		.owns(dp.clone(), Default::default())
		.owns(sec.clone(), Default::default())
		.shutdown_on_signal()
		.run(
			srv_reconcile,
			srv_error_policy,
			Arc::new(Data {
				client: client.clone(),
			}),
		)
		.for_each(|_| futures::future::ready(()));

	let cli_con = Controller::new(cli, Default::default())
		.owns(dp, Default::default())
		.owns(sec, Default::default())
		.shutdown_on_signal()
		.run(
			cli_reconcile,
			cli_error_policy,
			Arc::new(Data { client: client }),
		)
		.for_each(|_| futures::future::ready(()));

	let _ = futures::join!(srv_con, cli_con);
}
