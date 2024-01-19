use crate::{
	crd::SecretType,
	rathole::{ClientConfig, ClientServiceConfig, Config},
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
use std::{
	collections::{BTreeMap, HashMap},
	io::ErrorKind,
	sync::Arc,
};
use tokio::time::Duration;

use crate::config::initialize_config;
use crate::crd::{Client as RH_Client, Server as RH_Server};

// Data we want access to in error/reconcile calls
struct Data {
	client: Client,
}

async fn srv_reconcile(obj: Arc<RH_Server>, ctx: Arc<Data>) -> Result<Action> {
	let scr = (*obj).to_config_secret();
	let dp = (*obj).to_deployment();

	let client = &ctx.client;
	let ns = obj
		.metadata
		.namespace
		.as_ref()
		.ok_or_else(|| Error::MissingObjectKey(".metadata.namespace"))?;
	let name = obj
		.metadata
		.name
		.as_ref()
		.ok_or_else(|| Error::MissingObjectKey(".metadata.namespace"))?;
	let dp_api = Api::<Deployment>::namespaced(client.clone(), ns);
	let scr_api = Api::<Secret>::namespaced(client.clone(), ns);

	let pp = PatchParams::apply("server.rathole.mrchypark.github.io");
	dp_api
		.patch(name, &pp, &Patch::Apply(&dp))
		.await
		.map_err(Error::DeplymentCreationFailed)?;
	scr_api
		.patch(name, &pp, &Patch::Apply(&scr))
		.await
		.map_err(Error::SecretCreationFailed)?;

	Ok(Action::requeue(Duration::from_secs(10)))
}

fn srv_error_policy(obj: Arc<RH_Server>, err: &Error, _ctx: Arc<Data>) -> Action {
	println!("server request fail: {}", obj.name_any());
	println!("{}", err);
	Action::requeue(Duration::from_secs(5))
}

async fn cli_reconcile(obj: Arc<RH_Client>, ctx: Arc<Data>) -> Result<Action> {
	let client = &ctx.client;
	let env = initialize_config();

	let ns = obj
		.metadata
		.namespace
		.as_ref()
		.ok_or_else(|| Error::MissingObjectKey(".metadata.namespace"))?;
	let svr_api = Api::<RH_Server>::namespaced(client.clone(), ns);
	let dp_api = Api::<Deployment>::namespaced(client.clone(), ns);
	let scr_api = Api::<Secret>::namespaced(client.clone(), ns);

	// server ref check
	// crd, deploy, sercret check
	// client status update

	// get server secret

	// update server secret
	// patch server secret

	// client.to_config_sercret

	svr_api
		.get(&obj.spec.server_ref)
		.await
		.map_err(Error::NoTargetServerConfig)?;

	dp_api
		.get(&obj.spec.server_ref)
		.await
		.map_err(Error::NoTargetServer)?;

	let mut cc: HashMap<String, ClientServiceConfig> = HashMap::new();

	for s in &obj.spec.services {
		match s.token.r#type {
			Some(SecretType::Reference) | None => match &s.token.secret_ref {
				Some(SecretRef) => {
					let t = scr_api
						.get(&s.token.secret_ref.unwrap().name)
						.await
						.map_err(Error::NoTargetToken)?;
					let dat: BTreeMap<String, ByteString> = t.data.clone().unwrap();

					cc. = Some(String::from_utf8(dat[&env.rathole_config_name].0.clone()).unwrap());
				},
				None => {
					s.token
						.secret_ref
						.ok_or(std::io::Error::new(
							ErrorKind::NotFound,
							"Secret type is set Reference, but no data on SecretRef.",
						))
						.map_err(|e| Error::DefaultError(e));
				},
			},
			Some(SecretType::Direct) => {},
		}
	}

	println!("{:#?}", obj.spec.services);

	let s = scr_api
		.get(&obj.spec.server_ref)
		.await
		.map_err(Error::NoServerConfigFound)?;

	let dat: BTreeMap<String, ByteString> = s.data.clone().unwrap();

	let mut ref_config =
		toml::from_str::<Config>(&String::from_utf8(dat[&env.rathole_config_name].0.clone()).unwrap())
			.unwrap();

	ref_config.add_services(obj.spec.services.clone());

	let ns = Secret {
		data: Some(BTreeMap::from([(
			env.rathole_config_name.clone(),
			ByteString(ref_config.into_bytes()),
		)])),
		..Default::default()
	};

	scr_api
		.patch(
			s.metadata
				.name
				.as_ref()
				.ok_or_else(|| Error::MissingObjectKey(".metadata.name"))?,
			&PatchParams::apply("server.rathole.mrchypark.github.io"),
			&Patch::Apply(&ns),
		)
		.await
		.map_err(Error::SecretCreationFailed)?;

	// let client_config = scr_api
	// 	.clone()
	// 	.patch(
	// 		client_config
	// 			.metadata
	// 			.name
	// 			.as_ref()
	// 			.ok_or_else(|| Error::MissingObjectKey(".metadata.name"))?,
	// 		&PatchParams::apply("server.rathole.mrchypark.github.io"),
	// 		&Patch::Apply(&client_config),
	// 	)
	// 	.await
	// 	.map_err(Error::SecretCreationFailed)?;

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
