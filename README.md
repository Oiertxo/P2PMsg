# Distributed P2P Messenger (Prototype)

A fully distributed, serverless messaging application built for privacy, resilience, and battery efficiency. This project creates a private overlay network where users communicate directly (Peer-to-Peer) without relying on central servers for message storage or routing.

> **Note:** This project is part of my engineering portfolio, exploring advanced concepts in decentralized systems, cryptography (Signal Protocol), and memory-safe programming.

## 🚀 Project Overview

The goal is to implement a robust decentralized communication tool that resists censorship and ensures data sovereignty. Unlike traditional client-server architectures, this application empowers users to own their identity and data.

To solve the classic "P2P battery drain" and "connectivity" problems, the architecture will implement a **Multi-Layered Discovery Strategy** and an **Adaptive Topology**.

### Key Engineering Features

  * **🛡️ Zero-Trust Security:**
      * **Double Ratchet Algorithm:** Ensures *Perfect Forward Secrecy* (PFS). Breaking a session key only compromises a single message, not the entire history.
      * **Proof of Work (PoW) Handshake:** Prevents spam and Sybil attacks. New contact requests require solving a client-side cryptographic puzzle (Hashcash) before the receiver accepts the connection.
  * **⚡ Hybrid Bootstrapping (Connectivity Layer):**
      * **Local (mDNS):** Automatic discovery of peers on the same LAN/Wi-Fi (Zero-config offline messaging).
      * **Global (DHT + DNS):** Uses DNS seeds for initial bootstrapping and Kademlia DHT for global routing.
  * **🔋 Adaptive Topology:** Nodes automatically switch between **"Client Mode"** (mobile/low-power, only fetches data) and **"Relay Mode"** (desktop/charging, supports the network routing).
  * **📬 Asynchronous Persistence (Blind Mailbox):** Uses the DHT as a temporary, encrypted "dead drop". Messages sent to offline users are stored (blindly) on random nodes and retrieved when the recipient returns online.
  * **🔑 Sovereign Identity:** Uses **BIP39** (12-word mnemonic phrases) to deterministically derive `Ed25519` keys, allowing account recovery without a central server.

## ✅ Implementation Status

This project is currently in the **Alpha Prototype** phase.

  * [x] **Core Setup:** Rust + Flutter environment integration via `flutter_rust_bridge`.
  * [x] **Identity Generation:** Secure `Ed25519` keypairs derived from **BIP39** seed phrases.
  * [x] **Local Node Start:** Successful instantiation of the `libp2p` swarm and listening on local transport.
  * [ ] **Peering & Discovery:**
      * [ ] Verify connection between two distinct nodes.
      * [ ] Implement mDNS (Local) and Kademlia (Global) logic.
  * [ ] **Messaging Layer:** GossipSub / Request-Response implementation.
  * [ ] **Advanced Security:** Integration of `vodozemac` (Ratchet) and PoW challenge logic.

## 🛠️ The Tech Stack

Leveraging the "Golden Stack" for high-performance cross-platform engineering, selecting best-in-class crates for each logical layer:

| Logical Layer | Technology / Crate | Technical Description |
| :--- | :--- | :--- |
| **Identity** | `bip39`, `ed25519-dalek` | Mnemonic generation and Elliptic Curve signatures for sovereign identity. |
| **Networking** | `libp2p` | Modular P2P networking stack (Swarm, Transport, Behaviour). |
| **Encryption** | `libp2p-noise` | **Transport Layer:** Noise Protocol handshake for encrypted connections. |
| | `vodozemac` | **Application Layer:** Double Ratchet implementation (Matrix/Olm standard). |
| **Discovery** | `libp2p-mdns`, `libp2p-kad` | Hybrid discovery via Multicast DNS (LAN) and Kademlia DHT (WAN). |
| **Persistence** | `rusqlite` | Encrypted local SQLite database for chat history and session keys. |
| **Anti-Spam** | `sha2` | Custom Proof-of-Work (Hashcash) logic for connection requests. |
| **Bridge** | `flutter_rust_bridge` | Zero-copy FFI binding generation between Dart and Rust. |
| **Runtime** | `tokio` | Asynchronous runtime for handling non-blocking network I/O. |

## 📂 Project Structure

The codebase will follow a modular architecture separating the UI from the heavy lifting:

```
├── lib/                 # Flutter UI Code (Dart)
│   ├── main.dart        # Entry point & UI Logic
│   └── bridge_generated # Auto-generated bindings
├── native/              # Rust Core Logic (Crate)
│   ├── src/
│   │   ├── api.rs       # Exposed API for Flutter
│   │   ├── identity.rs  # BIP39 & Key Management
│   │   ├── network.rs   # libp2p Swarm, Behaviour & Events
│   │   ├── crypto.rs    # PoW & Ratchet logic (Placeholder)
│   │   └── main.rs      # Node entry point
│   └── Cargo.toml       # Rust dependencies
```

## 🚦 Getting Started

### Prerequisites

  * Flutter SDK
  * Rust Toolchain (Cargo)

### Running the App

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/oiera/p2p-messenger.git
    cd p2p-messenger
    ```
2.  **Install dependencies:**
    ```bash
    flutter pub get
    ```
3.  **Run generation script (for Rust bridge):**
    ```bash
    flutter_rust_bridge_codegen generate
    ```
4.  **Launch:**
    ```bash
    flutter run
    ```
    *Check the console output to see your generated PeerID\!*