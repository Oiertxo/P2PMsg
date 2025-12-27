import 'package:flutter/material.dart';
import '../../main.dart';

class PeerListView extends StatelessWidget {
  final Function(String) onPeerSelected;

  const PeerListView({super.key, required this.onPeerSelected});

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: nodeManager,
      builder: (context, _) {
        final peers = nodeManager.peers;

        if (peers.isEmpty) {
          return Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Icon(Icons.radar, size: 80, color: Colors.blue.withOpacity(0.3)),
                const SizedBox(height: 20),
                const Text(
                  "Scanning for peers...",
                  style: TextStyle(fontSize: 18, color: Colors.grey),
                ),
                const SizedBox(height: 10),
                const Text(
                  "Ensure devices are on the same WiFi\nor configure a Bootstrap Node in Settings.",
                  textAlign: TextAlign.center,
                  style: TextStyle(fontSize: 12, color: Colors.grey),
                ),
              ],
            ),
          );
        }

        return ListView.builder(
          padding: const EdgeInsets.all(8),
          itemCount: peers.length,
          itemBuilder: (context, index) {
            final peerId = peers[index];
            final unreadCount = nodeManager.unreadCount(peerId);
            final avatarColor = Colors.primaries[peerId.hashCode % Colors.primaries.length];
            final isOnline = nodeManager.isPeerOnline(peerId);

            return Card(
              elevation: 2,
              shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
              margin: const EdgeInsets.symmetric(vertical: 4),
              child: ListTile(
                contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
                leading: Stack(
                  children: [
                    CircleAvatar(
                      backgroundColor: avatarColor,
                      radius: 25,
                      child: const Icon(Icons.person, color: Colors.white),
                    ),
                    Positioned(
                      right: 0,
                      bottom: 0,
                      child: Container(
                        width: 12,
                        height: 12,
                        decoration: BoxDecoration(
                          color: isOnline ? Colors.green : Colors.grey,
                          shape: BoxShape.circle,
                          border: Border.all(color: Colors.white, width: 2),
                        ),
                      ),
                    ),
                  ],
                ),
                title: Text(
                  "Peer ${peerId.substring(0, 5)}...",
                  style: const TextStyle(fontWeight: FontWeight.bold, fontSize: 16),
                ),
                subtitle: Text(
                  peerId,
                  style: TextStyle(fontSize: 10, color: Colors.grey[600]),
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
                trailing: unreadCount > 0
                    ? Container(
                        padding: const EdgeInsets.all(8),
                        decoration: const BoxDecoration(
                          color: Colors.redAccent,
                          shape: BoxShape.circle,
                        ),
                        child: Text(
                          unreadCount.toString(),
                          style: const TextStyle(color: Colors.white, fontWeight: FontWeight.bold),
                        ),
                      )
                    : const Icon(Icons.chevron_right, color: Colors.grey),
                onTap: () => onPeerSelected(peerId),
              ),
            );
          },
        );
      },
    );
  }
}
