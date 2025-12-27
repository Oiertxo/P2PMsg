import 'package:flutter/material.dart';
import '../../main.dart';
import 'settings_screen.dart';
import '../widgets/chat_view.dart';
import '../widgets/peer_list_view.dart';

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> with WidgetsBindingObserver {
  String? _selectedPeer;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      nodeManager.refreshNode();
    }
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: nodeManager,
      builder: (context, child) {
        final bool showChat = _selectedPeer != null;

        return Scaffold(
          appBar: AppBar(
            title: Text(showChat ? "Chat with $_selectedPeer" : "P2P Chat"),
            leading: showChat
                ? IconButton(
                    icon: const Icon(Icons.arrow_back),
                    onPressed: () {
                      setState(() {
                        _selectedPeer = null;
                        nodeManager.activeChatPeerId = null;
                      });
                    },
                  )
                : null,
            actions: [
              if (!showChat) ...[
                // --- REFRESH BUTTON ---
                IconButton(
                  icon: const Icon(Icons.refresh),
                  tooltip: "Refresh Network",
                  onPressed: () {
                    nodeManager.refreshNode();

                    // Visual feedback for the user
                    ScaffoldMessenger.of(context).showSnackBar(
                      const SnackBar(
                      content: Text("Discovering peers..."),
                      duration: Duration(seconds: 1),
                      ),
                    );
                  },
                ),

                // --- SETTINGS BUTTON ---
                IconButton(
                  icon: const Icon(Icons.settings),
                  tooltip: "Network Settings",
                  onPressed: () {
                    Navigator.push(
                      context,
                      MaterialPageRoute(builder: (context) => SettingsScreen()),
                    );
                  },
                ),
              ],
            ],
          ),
          body: showChat
              ? ChatView(
                  peerId: _selectedPeer!,
                  onBack: () => setState(() => _selectedPeer = null),
                )
              : PeerListView(
                  onPeerSelected: (peerId) {
                    setState(() {
                      _selectedPeer = peerId;
                      nodeManager.activeChatPeerId = peerId;
                      nodeManager.markAsRead(peerId);
                    });
                  },
                ),
        );
      },
    );
  }
}
