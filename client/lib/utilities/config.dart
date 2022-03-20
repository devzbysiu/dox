import 'package:dox/utilities/log.dart';
import 'package:flutter_dotenv/flutter_dotenv.dart';

abstract class Config {
  String get baseUrl;
  String get websocketUrl;
}

class ConfigImpl with Log implements Config {
  static Future<Config> init() async {
    const env = String.fromEnvironment('ENV', defaultValue: 'simulator');
    _singleton ??= await ConfigImpl._fromEnv(env);
    return _singleton!;
  }

  static Future<Config> _fromEnv(String env) async {
    await dotenv.load(fileName: '.$env.env');
    return ConfigImpl._(dotenv.env['BASE_URL']!, dotenv.env['WEBSOCKET_URL']!);
  }

  ConfigImpl._(String baseUrl, String websocketUrl) {
    log.fine('initializing config');
    log.fine('\tbaseUrl: "$baseUrl"');
    log.fine('\twebsocketUrl: "$websocketUrl"');
    _coreBaseUrl = baseUrl;
    _coreWebSocketUrl = websocketUrl;
  }

  static Config? _singleton;

  late final String _coreBaseUrl;

  late final String _coreWebSocketUrl;

  @override
  String get baseUrl => _coreBaseUrl;

  @override
  String get websocketUrl => _coreWebSocketUrl;
}
