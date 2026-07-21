//! tepra binary entry point.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context as _;
use clap::Parser as _;
use tepra_core::otel::metrics::Meters;
use tepra_web::cli::{Cli, Commands, ConfigAction};
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
        Commands::Config(args) => match args.action {
            ConfigAction::Init(init) => {
                tepra_web::config::write_default_toml(&init.path, init.force)?;
                #[allow(clippy::print_stdout)]
                {
                    println!("wrote {}", init.path.display());
                }
            }
        },
        Commands::Serve(args) => {
            let (cfg, config_file_path) = tepra_web::config::load_config(&args)?;

            let telemetry =
                tepra_core::otel::init_telemetry(env!("CARGO_PKG_NAME"), env!("GIT_HASH"))
                    .context("failed to initialise telemetry")?;

            let config_file_display = config_file_path
                .as_ref()
                .map_or_else(|| "none".to_owned(), |p| p.display().to_string());
            tracing::info!(
                target: "tepra.config",
                template_dir = %cfg.template_dir.display(),
                bind = %cfg.bind,
                creator_base = %cfg.creator_base,
                config_file = %config_file_display,
                "loaded",
            );

            let bind = cfg.bind;
            let creator_base = cfg.creator_base;
            let template_dir = cfg.template_dir;

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
