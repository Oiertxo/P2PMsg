use libp2p::{
    core::{muxing::StreamMuxerBox, transport::Boxed},
    futures::future::Either,
    identity, noise, relay, tcp, yamux, PeerId, Transport,
};

// Build hybrid transport (TCP + Relay)
pub fn build_transport(
    keypair: &identity::Keypair,
    peer_id: PeerId,
) -> (Boxed<(PeerId, StreamMuxerBox)>, relay::client::Behaviour) {
    
    // Create relay client
    let (relay_transport, relay_client) = relay::client::new(peer_id);
    
    // Relay transport
    let relay_transport = relay_transport
        .upgrade(libp2p::core::upgrade::Version::V1Lazy)
        .authenticate(noise::Config::new(keypair).unwrap())
        .multiplex(yamux::Config::default())
        .boxed();

    // TCP transport
    let tcp_transport = tcp::tokio::Transport::default()
        .upgrade(libp2p::core::upgrade::Version::V1Lazy)
        .authenticate(noise::Config::new(keypair).unwrap())
        .multiplex(yamux::Config::default())
        .boxed();

    // Combination
    let transport = relay_transport
        .or_transport(tcp_transport)
        .map(|either, _| match either {
            Either::Left((peer_id, muxer)) => (peer_id, muxer),
            Either::Right((peer_id, muxer)) => (peer_id, muxer),
        })
        .boxed();

    (transport, relay_client)
}