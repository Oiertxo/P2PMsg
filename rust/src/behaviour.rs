use libp2p::{
    gossipsub, identify, kad::{store::MemoryStore, Behaviour as Kademlia}, 
    mdns::tokio::Behaviour as Mdns, ping::Behaviour as Ping, 
    relay, dcutr, swarm::NetworkBehaviour
};

#[derive(NetworkBehaviour)]
pub struct MyP2PBehaviour {
    pub kademlia: Kademlia<MemoryStore>,
    pub ping: Ping,
    pub mdns: Mdns,
    pub gossipsub: gossipsub::Behaviour,
    pub relay_client: relay::client::Behaviour,
    pub relay_server: relay::Behaviour,
    pub dcutr: dcutr::Behaviour,
    pub identify: identify::Behaviour,
}