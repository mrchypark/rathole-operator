use kube::CustomResourceExt;
use crate::crd::{Server as RH_Server, Client as RH_Client};
pub mod crd;

fn main() {
    print!("{}", serde_yaml::to_string(&RH_Client::crd()).unwrap());

    println!("---");
    
    print!("{}", serde_yaml::to_string(&RH_Server::crd()).unwrap());
}