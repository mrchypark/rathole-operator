use kube::{client::Client, Api, api::Patch, api::PatchParams};
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;

use crate::crd::{Server, Client};
pub mod crd;

#[tokio::main]
async fn main() -> anyhow::Result<()>{

  tracing_subscriber::fmt::init();

  let client: Client = Client::try_default()
    .await
    .expect("hi!");

  let rh_client = PatchParams::apply("client_apply").force();
  let crdClient: Api<CustomResourceDefinition> = Api::all(client.clone());

  crdClient.patch(crdName, rh_client, &Patch::Apply(&Client::crd()));

  Ok(())    
}
