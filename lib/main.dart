import 'package:flutter/material.dart';
import 'package:p2p_msg/src/rust/frb_generated.dart';
import 'package:p2p_msg/logic/node_manager.dart';

Future<void> main(List<String> args) async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  final instanceId = args.isNotEmpty ? args[0] : 'default';
  await NodeManager().start(instanceId);
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        useMaterial3: true, 
        colorSchemeSeed: Colors.indigo
      ),
      home: const MainScreen(),
    );
  }
}

class MainScreen extends StatefulWidget {
  const MainScreen({super.key});

  @override
  State<MainScreen> createState() => _MainScreenState();
}

class _MainScreenState extends State<MainScreen> {
  String? _selectedPeer;

  @override
  Widget build(BuildContext context) {
    // AnimatedBuilder listens to NodeManager changes and rebuilds this widget
    return AnimatedBuilder(
      animation: NodeManager(),
      builder: (context, child) {
        final manager = NodeManager();

        // If the selected peer disconnected, close the chat
        if (_selectedPeer != null && !manager.peers.contains(_selectedPeer)) {
          // Schedule state change for the next frame to avoid build errors
          WidgetsBinding.instance.addPostFrameCallback((_) {
            if (mounted) {
              setState(() {
                _selectedPeer = null;
                manager.activeChatPeerId = null;
              });
              ScaffoldMessenger.of(context).showSnackBar(const SnackBar(content: Text("User disconnected")));
            }
          });
        }

        // Conditional Navigation: Show Chat or List
        return Scaffold(
          body: _selectedPeer != null 
              ? ChatView(
                  peerId: _selectedPeer!, 
                  onBack: () {
                    setState(() => _selectedPeer = null);
                    manager.activeChatPeerId = null;
                  }
                )
              : PeerListView(
                  onPeerSelected: (peerId) {
                    setState(() => _selectedPeer = peerId);
                    manager.activeChatPeerId = peerId;
                    manager.markAsRead(peerId);
                  },
                ),
        );
      },
    );
  }
}

// Peer list
class PeerListView extends StatelessWidget {
  final Function(String) onPeerSelected;

  const PeerListView({super.key, required this.onPeerSelected});

  @override
  Widget build(BuildContext context) {
    final manager = NodeManager(); // Access data directly

    return Scaffold(
      appBar: AppBar(title: const Text("P2P Messenger")),
      body: Padding(
        padding: const EdgeInsets.all(20.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            // Identity Card
            Card(
              color: Colors.blue.shade50,
              child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: Column(
                  children: [
                    const Icon(Icons.person_pin, size: 40, color: Colors.blue),
                    const SizedBox(height: 5),
                    const Text("My Peer ID", style: TextStyle(fontWeight: FontWeight.bold)),
                    SelectableText(manager.myPeerId, style: const TextStyle(fontFamily: 'monospace', fontSize: 12)),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 20),
            const Divider(),
            const Text("Nearby devices (mDNS)", style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
            const SizedBox(height: 10),
            
            // List of Peers
            Expanded(
              child: manager.peers.isEmpty
                  ? const Center(child: Text("Scanning for peers...", style: TextStyle(color: Colors.grey)))
                  : ListView.builder(
                      itemCount: manager.peers.length,
                      itemBuilder: (context, index) {
                        final peerId = manager.peers[index];
                        final hasUnread = manager.unreadPeers.contains(peerId);

                        return Card(
                          child: ListTile(
                            leading: Stack(
                              children: [
                                const Icon(Icons.account_circle, size: 40, color: Colors.grey),
                                if (hasUnread)
                                  Positioned(
                                    right: 0, top: 0,
                                    child: Container(width: 12, height: 12, decoration: const BoxDecoration(color: Colors.red, shape: BoxShape.circle)),
                                  )
                              ],
                            ),
                            title: Text(peerId, maxLines: 1, overflow: TextOverflow.ellipsis),
                            subtitle: Text(
                              hasUnread ? "New message!" : "Tap to chat",
                              style: TextStyle(
                                color: hasUnread ? Colors.green : Colors.grey,
                                fontWeight: hasUnread ? FontWeight.bold : FontWeight.normal
                              )
                            ),
                            trailing: const Icon(Icons.chevron_right),
                            onTap: () => onPeerSelected(peerId),
                          ),
                        );
                      },
                    ),
            ),
          ],
        ),
      ),
    );
  }
}

// Chat Screen
class ChatView extends StatefulWidget {
  final String peerId;
  final VoidCallback onBack;

  const ChatView({super.key, required this.peerId, required this.onBack});

  @override
  State<ChatView> createState() => _ChatViewState();
}

class _ChatViewState extends State<ChatView> {
  final TextEditingController _textController = TextEditingController();
  final ScrollController _scrollController = ScrollController();

  @override
  void initState() {
    super.initState();
    // Mark as read again just in case a message arrived while opening
    NodeManager().markAsRead(widget.peerId); 
  }

  void _sendMessage() {
    NodeManager().sendMessageTo(widget.peerId, _textController.text);
    _textController.clear();
  }

  void _scrollToBottom() {
    if (_scrollController.hasClients) {
        _scrollController.animateTo(
          _scrollController.position.maxScrollExtent + 100, // Add a bit of buffer
          duration: const Duration(milliseconds: 300),
          curve: Curves.easeOut,
        );
    }
  }

  @override
  Widget build(BuildContext context) {
    // Access messages from the manager
    final messages = NodeManager().chatHistory[widget.peerId] ?? [];
    
    // Auto-scroll when new messages arrive
    WidgetsBinding.instance.addPostFrameCallback((_) => _scrollToBottom());

    return WillPopScope(
      onWillPop: () async {
        widget.onBack();
        return false;
      },
      child: Scaffold(
        appBar: AppBar(
          leading: IconButton(icon: const Icon(Icons.arrow_back), onPressed: widget.onBack),
          title: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text("Chat", style: TextStyle(fontSize: 14)),
              Text(widget.peerId, style: const TextStyle(fontSize: 10, fontWeight: FontWeight.normal)),
            ],
          ),
        ),
        body: Column(
          children: [
            // Message List
            Expanded(
              child: ListView.builder(
                controller: _scrollController,
                padding: const EdgeInsets.all(10),
                itemCount: messages.length,
                itemBuilder: (context, index) {
                  final msg = messages[index];
                  final isMe = msg['sender'] == 'me';
                  return Align(
                    alignment: isMe ? Alignment.centerRight : Alignment.centerLeft,
                    child: Container(
                      margin: const EdgeInsets.symmetric(vertical: 4, horizontal: 8),
                      padding: const EdgeInsets.all(12),
                      decoration: BoxDecoration(
                        color: isMe ? Colors.blue[100] : Colors.white,
                        borderRadius: BorderRadius.circular(12),
                        boxShadow: [BoxShadow(color: Colors.black12, blurRadius: 2, offset: const Offset(0, 1))]
                      ),
                      child: Text(msg['text'] ?? '', style: const TextStyle(fontSize: 16)),
                    ),
                  );
                },
              ),
            ),
            // Input Area
            Container(
              padding: const EdgeInsets.all(8),
              color: Colors.white,
              child: SafeArea(
                child: Row(
                  children: [
                    Expanded(
                      child: TextField(
                        controller: _textController,
                        decoration: InputDecoration(
                          hintText: "Write a message...",
                          filled: true, fillColor: Colors.grey[100],
                          border: OutlineInputBorder(borderRadius: BorderRadius.circular(25), borderSide: BorderSide.none),
                          contentPadding: const EdgeInsets.symmetric(horizontal: 20, vertical: 10),
                        ),
                        onSubmitted: (_) => _sendMessage(),
                      ),
                    ),
                    const SizedBox(width: 8),
                    FloatingActionButton(
                      mini: true,
                      onPressed: _sendMessage,
                      backgroundColor: Colors.blue,
                      child: const Icon(Icons.send, color: Colors.white, size: 20),
                    ),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}