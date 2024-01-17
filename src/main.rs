use actix_web::{get, middleware, App, HttpRequest, HttpResponse, HttpServer, Responder};
use rathole_operator::{config::initialize_config, controller, telemetry};
use serde::Serialize;

#[get("/health")]
async fn health(_: HttpRequest) -> impl Responder {
	HttpResponse::Ok().json("healthy")
}

#[derive(Serialize)]
struct StatusResponse {
	status: u16,
}

#[get("/")]
async fn index(_req: HttpRequest) -> impl Responder {
	let response = StatusResponse { status: 200 };
	HttpResponse::Ok().json(&response)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	telemetry::init().await;
	let env = initialize_config();

	let controller = controller::run();

	// Start web server
	let server = HttpServer::new(move || {
		App::new()
			.wrap(middleware::Logger::default().exclude("/health"))
			.service(index)
			.service(health)
	})
	.bind(format!("{}:{}", env.host, env.port))?
	.shutdown_timeout(5);

	// Both runtimes implements graceful shutdown, so poll until both are done
	tokio::join!(controller, server.run()).1?;
	Ok(())
}
