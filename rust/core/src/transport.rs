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

    // QUIC transport
    let quic_transport = libp2p::quic::tokio::Transport::new(libp2p::quic::Config::new(keypair))
        .map(|(peer_id, muxer), _| (peer_id, StreamMuxerBox::new(muxer)))
        .boxed();

    // Combination 1
    let tcp_plus_relay = tcp_transport
        .or_transport(relay_transport)
        .map(|either, _| match either {
            Either::Left(res) => res,
            Either::Right(res) => res,
        });

    // Combination 2
    let transport = quic_transport
        .or_transport(tcp_plus_relay)
        .map(|either, _| match either {
            Either::Left(res) => res,
            Either::Right(res) => res,
        })
        .boxed();

    (transport, relay_client)
}
