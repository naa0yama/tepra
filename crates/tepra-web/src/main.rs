//! tepra binary entry point.

use std::sync::Arc;

use anyhow::Context as _;
use clap::Parser as _;
use tepra_web::cli::{Cli, Commands};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Version => {
            #[allow(clippy::print_stdout)]
            {
                println!("{}", tepra_web::app_version());
            }
        }
        Commands::Serve(args) => {
            let _telemetry = tepra_core::otel::init_telemetry(env!("GIT_HASH"))
                .context("failed to initialise telemetry")?;

            let client = Arc::new(tepra_core::client::ReqwestTepraClient::new(
                args.creator_base,
            ));
            let state =
                tepra::state::AppState::new_with_template_dir(client.clone(), args.template_dir);

            let router = tepra::router::build_router(client)
                .merge(tepra::router::build_jobs_router(state.clone()))
                .merge(tepra::router::build_templates_router(state.clone()))
                .merge(tepra::router::build_ui_router(state))
                .merge(tepra_web::assets::router())
                .layer(TraceLayer::new_for_http());

            let listener = tokio::net::TcpListener::bind(&args.bind)
                .await
                .with_context(|| format!("failed to bind to {}", args.bind))?;
            axum::serve(listener, router)
                .await
                .context("server error")?;
        }
    }
    Ok(())
}
