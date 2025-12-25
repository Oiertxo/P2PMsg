import 'package:flutter/material.dart';
import 'package:p2pmsg/src/rust/api/node.dart';
import 'package:p2pmsg/src/rust/frb_generated.dart';

Future<void> main() async {
  // Initialize Rust
  await RustLib.init();
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: const Text('P2P Messenger')),
        body: Center(
          child: IdentityWidget(),
        ),
      ),
    );
  }
}

class IdentityWidget extends StatefulWidget {
  const IdentityWidget({super.key});

  @override
  State<IdentityWidget> createState() => _IdentityWidgetState();
}

class _IdentityWidgetState extends State<IdentityWidget> {
  String _myPeerId = "Generating keys...";
  final List<String> _peers = [];

  @override
  void initState() {
    super.initState();
    _startNodeAndListen();
  }

  void _startNodeAndListen() async {
    final stream = startP2PNode();

    stream.listen((message) {
      if (!mounted) return;
      
      setState(() {
        if (message.startsWith("ME:")) {
          // Self Peer ID
          _myPeerId = message.substring(3);
        } else if (message.startsWith("PEER+:")) {
          // New Peer connected
          final newPeer = message.substring(6);
          if (!_peers.contains(newPeer)) {
            _peers.add(newPeer);
          }
        } else if (message.startsWith("PEER-:")) {
          // Peer disconnected
          final lostPeer = message.substring(6);
          _peers.remove(lostPeer);
        }
      });
    });
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(20.0),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Identity card
          Card(
            color: Colors.blue.shade50,
            child: Padding(
              padding: const EdgeInsets.all(16.0),
              child: Column(
                children: [
                  const Icon(Icons.person_pin, size: 50, color: Colors.blue),
                  const SizedBox(height: 10),
                  const Text("My Peer ID", style: TextStyle(fontWeight: FontWeight.bold)),
                  SelectableText(_myPeerId, style: const TextStyle(fontFamily: 'monospace')),
                ],
              ),
            ),
          ),
          
          const SizedBox(height: 20),
          const Divider(),
          const Text("Nearby devices (mDNS)", style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
          const SizedBox(height: 10),

          // Detected Peer list
          Expanded(
            child: _peers.isEmpty
                ? const Center(child: Text("Looking for peers...", style: TextStyle(color: Colors.grey)))
                : ListView.builder(
                    itemCount: _peers.length,
                    itemBuilder: (context, index) {
                      return ListTile(
                        leading: const Icon(Icons.wifi_tethering, color: Colors.green),
                        title: Text("Peer detected"),
                        subtitle: Text(_peers[index], style: const TextStyle(fontSize: 12)),
                        trailing: const Icon(Icons.chat_bubble_outline),
                      );
                    },
                  ),
          ),
        ],
      ),
    );
  }
}