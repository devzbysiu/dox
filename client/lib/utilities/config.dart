import 'package:dox/utilities/log.dart';
import 'package:flutter_dotenv/flutter_dotenv.dart';

class Config with Log {
  static Config? _singleton;

  late final String _coreBaseUrl;

  late final String _coreWebSocketUrl;

  static Future<Config> init() async {
    const env = String.fromEnvironment('ENV', defaultValue: 'simulator');
    _singleton ??= await Config._fromEnv(env);
    return _singleton!;
  }

  static Future<Config> _fromEnv(String env) async {
    await dotenv.load(fileName: '.$env.env');
    return Config._(dotenv.env['BASE_URL']!, dotenv.env['WEBSOCKET_URL']!);
  }

  Config._(String baseUrl, String websocketUrl) {
    log.fine('initializing config');
    log.fine('\tbaseUrl: "$baseUrl"');
    log.fine('\twebsocketUrl: "$websocketUrl"');
    _coreBaseUrl = baseUrl;
    _coreWebSocketUrl = websocketUrl;
  }

  String get baseUrl => _coreBaseUrl;

  String get websocketUrl => _coreWebSocketUrl;
}
