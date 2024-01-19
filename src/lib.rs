use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
	#[error("GenkeyError: {0}")]
	GenkeyError(#[source] snowstorm::snow::Error),

	#[error("SerializationError: {0}")]
	SerializationError(#[source] serde_json::Error),

	#[error("Kube Error: {0}")]
	KubeError(#[source] kube::Error),

	#[error("Finalizer Error: {0}")]
	// NB: awkward type because finalizer::Error embeds the reconciler error (which is this)
	// so boxing this error to break cycles
	FinalizerError(#[source] Box<kube::runtime::finalizer::Error<Error>>),

	#[error("Failed to create ConfigMap: {0}")]
	ConfigMapCreationFailed(#[source] kube::Error),

	#[error("Failed to create Secret: {0}")]
	SecretCreationFailed(#[source] kube::Error),

	#[error("Failed to create Deplyment: {0}")]
	DeplymentCreationFailed(#[source] kube::Error),

	#[error("Failed to find server config: {0}")]
	NoServerConfigFound(#[source] kube::Error),

	#[error("Failed to get rathole server for use: {0}")]
	NoTargetServer(#[source] kube::Error),

	#[error("Failed to get server config: {0}")]
	NoTargetServerConfig(#[source] kube::Error),

	#[error("Failed to get Service Token Secret: {0}")]
	NoTargetToken(#[source] kube::Error),

	#[error("MissingObjectKey: {0}")]
	MissingObjectKey(&'static str),

	#[error("Error: {0}")]
	DefaultError(#[source] std::io::Error),
}
pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Error {
	pub fn metric_label(&self) -> String {
		format!("{self:?}").to_lowercase()
	}
}

/// Expose all controller components used by main
pub mod controller;
pub use crate::controller::*;

pub mod config;
pub mod crd;
pub mod method;
pub mod rathole;
/// Log and trace integrations
pub mod telemetry;
