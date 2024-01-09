use futures::StreamExt;
use kube::{
	runtime::controller::{Action, Controller},
	Api, Client, ResourceExt,
};
use std::{sync::Arc, time::Duration};

use crate::crd::{Client as RH_Client, Server as RH_Server};
pub mod crd;

#[derive(thiserror::Error, Debug)]
pub enum Error {}
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[tokio::main]
async fn main() -> Result<(), kube::Error> {
	let client = Client::try_default().await?;
	let srv: Api<RH_Server> = Api::all(client.clone());
	let cli: Api<RH_Client> = Api::all(client.clone());

	let srv_con = Controller::new(srv.clone(), Default::default())
		.run(srv_reconcile, srv_error_policy, Arc::new(()))
		.for_each(|_| futures::future::ready(()));

	let cli_con = Controller::new(cli.clone(), Default::default())
		.run(cli_reconcile, cli_error_policy, Arc::new(()))
		.for_each(|_| futures::future::ready(()));

	futures::join!(srv_con, cli_con);

	Ok(())
}

async fn srv_reconcile(obj: Arc<RH_Server>, _ctx: Arc<()>) -> Result<Action> {
	println!("server request: {}", obj.name_any());
	Ok(Action::requeue(Duration::from_secs(10)))
}

fn srv_error_policy(_object: Arc<RH_Server>, _err: &Error, _ctx: Arc<()>) -> Action {
	Action::requeue(Duration::from_secs(5))
}

async fn cli_reconcile(obj: Arc<RH_Client>, _ctx: Arc<()>) -> Result<Action> {
	println!("client request: {}", obj.name_any());
	Ok(Action::requeue(Duration::from_secs(10)))
}

fn cli_error_policy(_object: Arc<RH_Client>, _err: &Error, _ctx: Arc<()>) -> Action {
	Action::requeue(Duration::from_secs(5))
}
