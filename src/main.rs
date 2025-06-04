use clap::Parser;
use tracing_subscriber::prelude::*;

use blockchain::args::Args;
use blockchain::node;

#[tokio::main]
async fn main() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_current_span(true)
        .with_file(true)
        .with_line_number(true);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    tracing::debug!(?args, "Blockchain node configuration");
    node::start::start_node(args)
        .await
        .expect("Failed to start a blockchain node");
}
