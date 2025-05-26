use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Address of an existing node to join the cluster
    #[arg(short, long)]
    pub seed_node: Option<std::net::SocketAddr>,
    /// Cluster sync period (in seconds)
    #[arg(long, default_value_t = 30)]
    pub cluster_sync_period: u64,
    #[arg(short, long)]
    /// Port to listen on for incoming connections.
    pub port: u16,
    /// Difficulty level for the Proof of Work algorithm
    #[arg(short, long, default_value_t = 2)]
    pub difficulty: usize,
}
