import 'package:flutter/material.dart';
import 'logic/node_manager.dart';
import 'ui/screens/home_screen.dart';
import 'src/rust/frb_generated.dart';

// Global instance
final nodeManager = NodeManager();

void main(List<String> args) async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  String instanceName = args.isNotEmpty ? args[0] : "default";
  await nodeManager.start(instanceName);
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'P2P Chat',
      theme: ThemeData(
        primarySwatch: Colors.blue,
        useMaterial3: true,
      ),
      home: const HomeScreen(),
    );
  }
}
