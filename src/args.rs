use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub seed_node: Option<std::net::SocketAddr>,
    #[arg(short, long)]
    pub port: u16,
    #[arg(short, long, default_value_t = 2)]
    pub difficulty: usize,
}
