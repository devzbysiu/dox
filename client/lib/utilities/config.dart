import 'package:dox/utilities/log.dart';
import 'package:flutter_dotenv/flutter_dotenv.dart';

abstract class Config {
  String get baseUrl;
}

class ConfigImpl with Log implements Config {
  static Future<Config> init() async {
    const env = String.fromEnvironment('ENV', defaultValue: 'simulator');
    _singleton ??= await ConfigImpl._fromEnv(env);
    return _singleton!;
  }

  static Future<Config> _fromEnv(String env) async {
    await dotenv.load(fileName: '.$env.env');
    return ConfigImpl._(dotenv.env['BASE_URL']!);
  }

  ConfigImpl._(String baseUrl) {
    log.fine('initializing config');
    log.fine('\tbaseUrl: "$baseUrl"');
    _coreBaseUrl = baseUrl;
  }

  static Config? _singleton;

  late final String _coreBaseUrl;

  @override
  String get baseUrl => _coreBaseUrl;
}
