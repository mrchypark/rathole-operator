use crate::Result;
use base64::{
	alphabet,
	engine::{self, general_purpose},
	Engine,
};

const DEFAULT_CURVE: KeypairType = KeypairType::X25519;

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
