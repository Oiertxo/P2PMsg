use std::time::Duration;
use futures::StreamExt;
use libp2p::{
    gossipsub,
    kad::{store::MemoryStore, Behaviour as Kademlia, Config as KademliaConfig},
    ping::{Behaviour as Ping, Config as PingConfig},
    mdns::{tokio::Behaviour as Mdns, Config as MdnsConfig},
    swarm::SwarmEvent,
    SwarmBuilder, PeerId, identify, relay, dcutr,
};
use p2p_core::identity::get_or_create_identity;
use p2p_core::behaviour::{MyP2PBehaviour, MyP2PBehaviourEvent};
use p2p_core::transport::build_transport;

#[tokio::main]
async fn main() {
    // Server config
    let listen_port = 4001;
    let storage_path = "./";
    let instance_name = "oracle_relay_v1";

    println!("Starting Relay Server P2P");

    // Identity
    let id_keys = get_or_create_identity(storage_path, instance_name);
    let peer_id = PeerId::from(id_keys.public());

    println!("-------------------------------------------------");
    println!("SERVER PEER ID: {}", peer_id);
    println!("-------------------------------------------------");

    // Transport
    let (transport, relay_client) = build_transport(&id_keys, peer_id);

    // Behaviour
    let mut kad_config = KademliaConfig::default();
    kad_config.set_protocol_names(vec![libp2p::StreamProtocol::new("/p2p_msg/kad/1.0.0")]);

    // Gossipsub
    let gossip_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(1))
        .validation_mode(gossipsub::ValidationMode::Strict)
        .build()
        .expect("Configuración de Gossipsub válida");

    let gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(id_keys.clone()),
        gossip_config,
    ).expect("Error creando Gossipsub");

    // Relay
    let relay_config = relay::Config {
        max_reservations: 1024,
        max_circuits: 1024,
        reservation_duration: Duration::from_secs(60 * 60),
        ..Default::default()
    };

    let behaviour = MyP2PBehaviour {
        kademlia: Kademlia::with_config(peer_id, MemoryStore::new(peer_id), kad_config),
        ping: Ping::new(PingConfig::new().with_interval(Duration::from_secs(30))),
        mdns: Mdns::new(MdnsConfig::default(), peer_id).expect("Error mDNS"),
        gossipsub,
        identify: identify::Behaviour::new(identify::Config::new(
            "/p2p_msg/id/1.0.0".to_string(),
            id_keys.public()
        )),
        relay_client,
        relay_server: relay::Behaviour::new(peer_id, relay_config),
        dcutr: dcutr::Behaviour::new(peer_id),
    };

    // Swarm
    let mut swarm = SwarmBuilder::with_existing_identity(id_keys)
        .with_tokio()
        .with_other_transport(|_| transport).expect("Error Transporte")
        .with_behaviour(|_| behaviour).expect("Error Behaviour")
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60 * 60)))
        .build();

    // Listening
    let tcp_addr = format!("/ip4/0.0.0.0/tcp/{}", listen_port).parse::<libp2p::Multiaddr>().unwrap();
    let udp_addr = format!("/ip4/0.0.0.0/udp/{}/quic-v1", listen_port).parse::<libp2p::Multiaddr>().unwrap();

    swarm.listen_on(tcp_addr).unwrap();
    swarm.listen_on(udp_addr).unwrap();

    // Subscribe to Gossipsub
    let topic = gossipsub::IdentTopic::new("p2p-chat-global");
    swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();
    println!("Server subscribed to topic: p2p-chat-global");

    println!("Server listening on port {}", listen_port);
    println!("Waiting for connections...");

    // Event loop
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on: {}", address);
            },

            // This is triggered the moment a TCP/QUIC socket connects
            SwarmEvent::IncomingConnection { send_back_addr, .. } => {
                println!(">>> [TRANSPORT] Incoming connection attempt from: {:?}", send_back_addr);
            },

            // If something fails during the handshake
            SwarmEvent::IncomingConnectionError { error, .. } => {
                println!(">>> [ERROR] Handshake failed: {:?}", error);
            },

            // This is triggered when libp2p successfully negotiates Noise/Yamux
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                println!(">>> [LIBP2P] Connection established with Peer: {}", peer_id);
                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
            },

            // Disconnection
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                println!("Disconnection: {}", peer_id);
                swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
            },

            // Relay events
            SwarmEvent::Behaviour(MyP2PBehaviourEvent::RelayServer(event)) => {
                match event {
                    relay::Event::ReservationReqAccepted { src_peer_id, .. } => {
                        println!("Reserve accepted for: {}", src_peer_id);
                    },
                    relay::Event::CircuitReqAccepted { src_peer_id, dst_peer_id, .. } => {
                        println!("Circuit created: {} <--> {}", src_peer_id, dst_peer_id);
                    },
                    relay::Event::ReservationReqDenied { src_peer_id } => {
                        println!("Reserve denied: {}", src_peer_id);
                    },
                    _ => {}
                }
            },

            // DEBUG: Ver mensajes que pasan por el servidor
            SwarmEvent::Behaviour(MyP2PBehaviourEvent::Gossipsub(gossipsub::Event::Message { message, .. })) => {
                let text = String::from_utf8_lossy(&message.data);
                println!("Relay forwarding msg: {}", text);
            },

            // Kademlia discovery event
            SwarmEvent::Behaviour(MyP2PBehaviourEvent::Identify(identify::Event::Received { peer_id, info, .. })) => {
                for addr in info.listen_addrs {
                    swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                }
                println!("Peer identified: {}", peer_id);
            },

            _ => {}
        }
    }
}
