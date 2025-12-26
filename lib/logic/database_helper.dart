import 'dart:io';
import 'package:path/path.dart';
import 'package:sqflite/sqflite.dart';
import 'package:sqflite_common_ffi/sqflite_ffi.dart';
import 'package:path_provider/path_provider.dart';

class DatabaseHelper {
  // Singleton Pattern: Only one instance of the DB connection allowed
  static final DatabaseHelper _instance = DatabaseHelper._internal();
  factory DatabaseHelper() => _instance;
  DatabaseHelper._internal();

  Database? _database;

  Future<void> init(String storagePath, String instanceName) async {
    if (_database != null) return;

    // Initialize FFI    
    if (Platform.isWindows || Platform.isLinux || Platform.isMacOS) {
      sqfliteFfiInit();
      databaseFactory = databaseFactoryFfi;
    }

    final path = join(storagePath, 'p2p_chat_$instanceName.db');
    print("DB Path: $path");

    _database = await openDatabase(
      path,
      version: 1,
      onCreate: (db, version) async {
        // Create the messages table
        await db.execute('''
          CREATE TABLE messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            peerId TEXT NOT NULL,
            sender TEXT NOT NULL,
            text TEXT NOT NULL,
            timestamp INTEGER NOT NULL
          )
        ''');
      },
    );
  }

  // Private getter
  Database _getDb() {
    if (_database == null) {
      throw Exception("Database not initialized! Call init() first.");
    }
    return _database!;
  }

  // Save a new message
  Future<int> insertMessage(String peerId, String sender, String text) async {
    final db = _getDb();
    return await db.insert('messages', {
      'peerId': peerId,
      'sender': sender,
      'text': text,
      'timestamp': DateTime.now().millisecondsSinceEpoch,
    });
  }

  // Retrieve all history sorted by time
  Future<List<Map<String, dynamic>>> getAllMessages() async {
    final db = _getDb();
    return await db.query('messages', orderBy: 'timestamp ASC');
  }
}