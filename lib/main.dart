import 'package:flutter/material.dart';
import 'package:p2pmsg/src/rust/api/node.dart';
import 'package:p2pmsg/src/rust/frb_generated.dart';

Future<void> main() async {
  await RustLib.init();
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
      home: Scaffold(
        appBar: AppBar(title: const Text('P2P Messenger')),
        body: const IdentityWidget(),
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
  // Peer list
  final List<String> _peers = [];

  // Chat history
  // Structure: { 'PeerID': [ { 'sender': 'me', 'text': 'hello' }, ... ] }
  final Map<String, List<Map<String, String>>> _chatHistory = {};

  // Unread Peers
  final Set<String> _unreadPeers = {};

  // App navigation
  String? _selectedPeer;

  // Text and scroll controllers
  final TextEditingController _textController = TextEditingController();
  final ScrollController _scrollController = ScrollController();

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
        // Self Peer ID
        if (message.startsWith("ME:")) {
          _myPeerId = message.substring(3);
        }
        // New Peer connected
        else if (message.startsWith("PEER+:")) {
          final newPeer = message.substring(6);
          if (!_peers.contains(newPeer)) {
            _peers.add(newPeer);
          }
        }
        // Peer disconnected
        else if (message.startsWith("PEER-:")) {
          final lostPeer = message.substring(6);
          _peers.remove(lostPeer);
          // Close chat
          if (_selectedPeer == lostPeer) {
            _selectedPeer = null;
            ScaffoldMessenger.of(context).showSnackBar(
              const SnackBar(content: Text("User disconnected"))
            );
          }
        }
        // Message received from Peer
        else if (message.startsWith("MSG:")) {
          final parts = message.split(":");
          if (parts.length >= 3) {
            final sender = parts[1];
            final text = parts.sublist(2).join(":");
            // Initialize history
            if (!_chatHistory.containsKey(sender)) {
              _chatHistory[sender] = [];
            }
            // Store message
            _chatHistory[sender]!.add({'sender': 'peer', 'text': text});

            // Notification
            if (_selectedPeer != sender) {
              _unreadPeers.add(sender);
            } else {
              _scrollToBottom();
            }
          }
        }
        // Rust sends own message
        else if (message.startsWith("MSG_SENT:")) {
          final parts = message.split(":");
          if (parts.length >= 3) {
            final recipientId = parts[1];
            final text = parts.sublist(2).join(":");
            // Search receiver's chat
            if (!_chatHistory.containsKey(recipientId)) {
              _chatHistory[recipientId] = [];
            }            
            // Add message
            _chatHistory[recipientId]!.add({'sender': 'me', 'text': text});
            // Scroll
            if (_selectedPeer == recipientId) {
              _scrollToBottom();
            }
          }
        }
      });
    });
  }

  void _sendMessage() {
    if (_selectedPeer == null) return;
    
    final text = _textController.text.trim();
    if (text.isEmpty) return;

    sendMessage(recipient: _selectedPeer!, msg: text); 
    _textController.clear();
  }

  void _scrollToBottom() {
    // Use a small delay to ensure the ListView has rendered the new item
    // before we try to scroll to it.
    Future.delayed(const Duration(milliseconds: 100), () {
      if (_scrollController.hasClients) {
        _scrollController.animateTo(
          _scrollController.position.maxScrollExtent,
          duration: const Duration(milliseconds: 300),
          curve: Curves.easeOut,
        );
      }
    });
  }

  // Navigation
  @override
  Widget build(BuildContext context) {
    if (_selectedPeer != null) {
      return _buildChatScreen();
    } else {
      return _buildPeerList();
    }
  }

  // Peer list
  Widget _buildPeerList() {
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
                      final peerId = _peers[index];
                      // Check for unread messages
                      final hasUnread = _unreadPeers.contains(peerId);

                      return Card(
                        margin: const EdgeInsets.symmetric(vertical: 4),
                        child: ListTile(
                          leading: Stack(
                            children: [
                              const Icon(Icons.account_circle, size: 40, color: Colors.grey),
                              // Red dot for unread messages
                              if (hasUnread) 
                                Positioned(
                                  right: 0, top: 0,
                                  child: Container(
                                    width: 12, height: 12,
                                    decoration: const BoxDecoration(color: Colors.red, shape: BoxShape.circle),
                                  ),
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
                          // Enter chat
                          onTap: () {
                            setState(() {
                              _selectedPeer = peerId;
                              _unreadPeers.remove(peerId); // Mark as read
                            });
                          },
                        ),
                      );
                    },
                  ),
          ),
        ],
      ),
    );
  }

  // Chat screen
Widget _buildChatScreen() {
    final messages = _chatHistory[_selectedPeer] ?? [];

    return WillPopScope(
      onWillPop: () async {
        setState(() { _selectedPeer = null; });
        return false; 
      },
      child: Column(
        children: [
          // Chat header
          Container(
            color: Colors.blue.shade50,
            padding: const EdgeInsets.symmetric(vertical: 8, horizontal: 8),
            child: Row(
              children: [
                IconButton(
                  icon: const Icon(Icons.arrow_back),
                  onPressed: () => setState(() { _selectedPeer = null; }),
                ),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const Text("Chatting with:", style: TextStyle(fontSize: 10, color: Colors.grey)),
                      Text(_selectedPeer!, style: const TextStyle(fontWeight: FontWeight.bold, overflow: TextOverflow.ellipsis)),
                    ],
                  ),
                )
              ],
            ),
          ),

          // Message list
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
                      borderRadius: BorderRadius.only(
                        topLeft: const Radius.circular(12),
                        topRight: const Radius.circular(12),
                        bottomLeft: isMe ? const Radius.circular(12) : Radius.zero,
                        bottomRight: isMe ? Radius.zero : const Radius.circular(12),
                      ),
                      boxShadow: [
                        BoxShadow(color: Colors.black.withOpacity(0.05), blurRadius: 2, offset: const Offset(0, 1))
                      ]
                    ),
                    child: Text(msg['text'] ?? '', style: const TextStyle(fontSize: 16)),
                  ),
                );
              },
            ),
          ),

          // Input
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
                        filled: true,
                        fillColor: Colors.grey[100],
                        border: OutlineInputBorder(
                          borderRadius: BorderRadius.circular(25),
                          borderSide: BorderSide.none,
                        ),
                        contentPadding: const EdgeInsets.symmetric(horizontal: 20, vertical: 10),
                      ),
                      textCapitalization: TextCapitalization.sentences,
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
    );
  }
}