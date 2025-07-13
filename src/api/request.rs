use crate::blockchain::Block;
use crate::transaction::Transaction;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum Request {
    Join(std::net::SocketAddr),
    PeerList(Vec<std::net::SocketAddr>),
    GetChain,
    SyncBlock(Block),
    Ping,
    AddTransaction(Transaction),
    SyncTransaction(Transaction),
}
