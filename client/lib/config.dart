import 'package:flutter_dotenv/flutter_dotenv.dart';

class Config {
  static Config? _singleton;

  late final String _coreBaseUrl;

  static Future<Config> init() async {
    const env = String.fromEnvironment('ENV', defaultValue: 'simulator');
    _singleton ??= await Config._fromEnv(env);
    return _singleton!;
  }

  static Future<Config> _fromEnv(String env) async {
    await dotenv.load(fileName: '.$env.env');
    return Config._(dotenv.env['BASE_URL']!);
  }

  Config._(String baseUrl) {
    _coreBaseUrl = baseUrl;
  }

  String get baseUrl => _coreBaseUrl;
}