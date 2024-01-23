use crate::{
	crd::SecretType,
	rathole::{ClientServiceConfig, Config, ServiceType},
	Error, Result,
};

use kube::{
	api::{Api, ObjectMeta, Patch, PatchParams},
	client::Client,
	core::object::HasSpec,
	runtime::{
		controller::{Action, Controller},
		watcher::Event,
	},
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

	svr_api
		.get(&obj.spec.server_ref)
		.await
		.map_err(Error::NoTargetServerConfig)?;

	dp_api
		.get(&obj.spec.server_ref)
		.await
		.map_err(Error::NoTargetServer)?;

	// get secretref to object
	let mut cc: HashMap<String, ClientServiceConfig> = HashMap::new();
	for s in &obj.spec.services {
		match s.token.r#type {
			Some(SecretType::Reference) | None => {
				match &s.token.secret_ref {
					Some(secret_ref) => {
						let secret = scr_api
							.get(&secret_ref.name)
							.await
							.map_err(Error::NoTargetToken)?;

						if let Some(data) = secret.data {
							if let Some(value) = data.get(&secret_ref.key) {
								let config_str = String::from_utf8(value.0.clone()).unwrap();
								cc.insert(s.name.clone(), s.to_config(config_str));
							}
						}
					},
					None => {
						// 여기에 Reference 타입이지만 secret_ref가 없는 경우의 처리를 추가합니다.
						return Err(Error::DefaultError(std::io::Error::new(
							ErrorKind::NotFound,
							"Secret type is set to Reference, but no data on SecretRef.",
						)));
					},
				}
			},
			Some(SecretType::Direct) => {
				cc.insert(s.name.clone(), s.to_config(s.token.key.clone().unwrap()));
			},
		}
	}

	let s = scr_api
		.get(&obj.spec.server_ref)
		.await
		.map_err(Error::NoServerConfigFound)?;

	let dat: BTreeMap<String, ByteString> = s.data.clone().unwrap();

	let mut ref_config =
		toml::from_str::<Config>(&String::from_utf8(dat[&env.rathole_config_name].0.clone()).unwrap())
			.unwrap();

	for s in cc.values() {
		ref_config.add_service(s.clone());
	}

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
		.for_each(|event| match event {
			Event::Applied(o) => {},
			Event::Deleted(o) => {},
			Event::Restarted(o) => {},
			

    /// An object was added or modified
    //Applied(K),
    /// An object was deleted
    ///
    /// NOTE: This should not be used for managing persistent state elsewhere, since
    /// events may be lost if the watcher is unavailable. Use Finalizers instead.
    // Deleted(K),
    /// The watch stream was restarted, so `Deleted` events may have been missed
    ///
    /// Should be used as a signal to replace the store contents atomically.
    ///
    /// Any objects that were previously [`Applied`](Event::Applied) but are not listed in this event
    /// should be assumed to have been [`Deleted`](Event::Deleted).
    // Restarted(Vec<K>),
		});

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
