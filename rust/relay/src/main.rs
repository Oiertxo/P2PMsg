use std::time::Duration;
use futures::StreamExt;
use libp2p::{
    gossipsub,
    kad::{store::MemoryStore, Behaviour as Kademlia, Config as KademliaConfig},
    ping::{Behaviour as Ping, Config as PingConfig},
    mdns::{tokio::Behaviour as Mdns, Config as MdnsConfig},
    swarm::{SwarmEvent, NetworkBehaviour},
    SwarmBuilder, PeerId, identify, relay,
};
use p2p_core::identity::get_or_create_identity;
use p2p_core::transport::build_transport;
use p2p_core::logger::init_p2p_logging;
use tracing::{info, warn};

// Relay behaviour
#[derive(NetworkBehaviour)]
struct RelayBehaviour {
    gossipsub: gossipsub::Behaviour,
    kademlia: Kademlia<MemoryStore>,
    ping: Ping,
    mdns: Mdns,
    identify: identify::Behaviour,
    relay: relay::Behaviour,
}

#[tokio::main]
async fn main() {
    // Server configuration
    let listen_port = 4001;
    let storage_path = "./";
    let instance_name = "oracle_relay_v1";

    // Identity
    let id_keys = get_or_create_identity(storage_path, instance_name);
    let peer_id = PeerId::from(id_keys.public());

    // Logging
    let _guard = init_p2p_logging(storage_path, &peer_id);

    info!("Starting Relay Server P2P on port {}", listen_port);
    info!("SERVER PEER ID: {}", peer_id);

    // Transport
    let (transport, _relay_client_transport) = build_transport(&id_keys, peer_id);

    // Behaviour configuration
    let mut kad_config = KademliaConfig::default();
    kad_config.set_protocol_names(vec![libp2p::StreamProtocol::new("/p2p_msg/kad/1.0.0")]);

    let gossip_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(1))
        .validation_mode(gossipsub::ValidationMode::Strict)
        .build()
        .expect("Error creating Gossipsub config");

    let gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(id_keys.clone()),
        gossip_config,
    ).expect("Error creating Gossipsub Behaviour");

    let relay_config = relay::Config {
        max_reservations: 1024,
        max_circuits: 1024,
        reservation_duration: Duration::from_secs(3600),
        ..Default::default()
    };

    // Relay Behaviour config
    let behaviour = RelayBehaviour {
        kademlia: Kademlia::with_config(peer_id, MemoryStore::new(peer_id), kad_config),
        ping: Ping::new(PingConfig::new().with_interval(Duration::from_secs(30))),
        mdns: Mdns::new(MdnsConfig::default(), peer_id).expect("mDNS Error"),
        gossipsub,
        identify: identify::Behaviour::new(identify::Config::new(
            "/p2p_msg/id/1.0.0".to_string(),
            id_keys.public()
        )),
        relay: relay::Behaviour::new(peer_id, relay_config),
    };

    // Swarm
    let mut swarm = SwarmBuilder::with_existing_identity(id_keys)
        .with_tokio()
        .with_other_transport(|_| transport).expect("Transport Error")
        .with_behaviour(|_| behaviour).expect("Behaviour Error")
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(3600)))
        .build();

    // Listeners
    let tcp_addr = format!("/ip4/0.0.0.0/tcp/{}", listen_port).parse::<libp2p::Multiaddr>().unwrap();
    let udp_addr = format!("/ip4/0.0.0.0/udp/{}/quic-v1", listen_port).parse::<libp2p::Multiaddr>().unwrap();

    swarm.listen_on(tcp_addr).unwrap();
    swarm.listen_on(udp_addr).unwrap();

    // Subscribe to global topic
    let topic = gossipsub::IdentTopic::new("p2p-chat-global");
    swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();
    info!("Server subscribed to topic: p2p-chat-global");

    // Event loop
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("Listening on: {}", address);
            },

            SwarmEvent::IncomingConnection { send_back_addr, .. } => {
                info!("New incoming connection from: {:?}", send_back_addr);
            },

            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                info!("Connection established with Peer: {}", peer_id);
                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
            },

            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                info!("Disconnection: {}", peer_id);
                swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
            },

            // Relay events
            SwarmEvent::Behaviour(RelayBehaviourEvent::Relay(event)) => {
                match event {
                    relay::Event::ReservationReqAccepted { src_peer_id, .. } => {
                        info!("Reservation accepted for: {}", src_peer_id);
                    },
                    relay::Event::CircuitReqAccepted { src_peer_id, dst_peer_id, .. } => {
                        info!("Circuit created: {} <--> {}", src_peer_id, dst_peer_id);
                    },
                    relay::Event::ReservationReqDenied { src_peer_id } => {
                        warn!("Reservation denied for: {}", src_peer_id);
                    },
                    relay::Event::CircuitReqDenied { src_peer_id, dst_peer_id } => {
                        warn!("Circuit denied: {} -> {}", src_peer_id, dst_peer_id);
                    },
                    _ => {}
                }
            },

            // Message Debugging
            SwarmEvent::Behaviour(RelayBehaviourEvent::Gossipsub(gossipsub::Event::Message { message, .. })) => {
                let text = String::from_utf8_lossy(&message.data);
                if !text.contains("REFRESH") {
                    info!("Relay forwarding msg: {}", text);
                }
            },

            // Kademlia / Identify
            SwarmEvent::Behaviour(RelayBehaviourEvent::Identify(identify::Event::Received { peer_id, info, .. })) => {
                for addr in info.listen_addrs {
                    swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                }
                info!("Peer identified: {}", peer_id);
            },

            _ => {}
        }
    }
}
