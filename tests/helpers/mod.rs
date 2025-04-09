use std::net::{SocketAddr, TcpListener};
use tokio::task::JoinHandle;

use blockchain::args::Args;
use blockchain::node;

pub struct TestServer {
    handle: JoinHandle<()>,
    pub addr: SocketAddr,
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.handle.abort()
    }
}

pub async fn start_node() -> TestServer {
    let addr = get_server_addr();
    let args = Args {
        seed_node: None,
        port: addr.port(),
        difficulty: 1,
    };
    let handle = tokio::spawn(async {
        node::start(args).await.expect("Could not start server");
    });
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    TestServer { handle, addr }
}

fn get_server_addr() -> SocketAddr {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("Failed to bind random port");
    listener.local_addr().expect("Failed to get local address")
}
