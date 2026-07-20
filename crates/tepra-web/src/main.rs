//! tepra binary entry point.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context as _;
use clap::Parser as _;
use tepra_core::otel::metrics::Meters;
use tepra_web::cli::{Cli, Commands};
use tepra_web::trace::{OtelHttpServerMakeSpan, OtelOnResponse, server_metrics_mw};
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
            // TODO(Cycle 9): replace all unwrap_or_else defaults with load_config() cascade.
            let template_dir = args
                .template_dir
                .unwrap_or_else(|| std::path::PathBuf::from("templates"));
            let bind = args.bind.unwrap_or_else(|| "0.0.0.0:3000".to_owned());
            let creator_base = args
                .creator_base
                .unwrap_or_else(|| "http://localhost:29108".to_owned());

            let telemetry =
                tepra_core::otel::init_telemetry(env!("CARGO_PKG_NAME"), env!("GIT_HASH"))
                    .context("failed to initialise telemetry")?;

            // Single shared Meters instance — client and server middleware share instruments.
            let meters = Arc::new(Meters::new());

            let client = Arc::new(tepra_core::client::ReqwestTepraClient::with_meters(
                creator_base,
                Arc::clone(&meters),
            ));
            let state = tepra::state::AppState::new_with_template_dir(client.clone(), template_dir);

            let router = tepra::router::build_router(client)
                .merge(tepra::router::build_jobs_router(state.clone()))
                .merge(tepra::router::build_templates_router(state.clone()))
                .merge(tepra::router::build_ui_router(state))
                .merge(tepra_web::assets::router())
                .layer(axum::middleware::from_fn_with_state(
                    Arc::clone(&meters),
                    server_metrics_mw,
                ))
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(OtelHttpServerMakeSpan)
                        .on_response(OtelOnResponse),
                );

            let listener = tokio::net::TcpListener::bind(&bind)
                .await
                .with_context(|| format!("failed to bind to {bind}"))?;
            axum::serve(
                listener,
                router.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .with_graceful_shutdown(async {
                tokio::signal::ctrl_c().await.ok();
            })
            .await
            .context("server error")?;

            telemetry.shutdown().await;
        }
    }
    Ok(())
}
