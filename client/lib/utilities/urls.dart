import 'package:dox/utilities/config.dart';
import 'package:dox/utilities/log.dart';
import 'package:dox/utilities/service_locator.dart';

class Urls with Log {
  late final Config _config;

  Urls({Config? config}) {
    log.fine('initializing Config');
    _config = config ?? getIt<Config>();
  }

  Uri search(String query) {
    return Uri.parse('${_config.baseUrl}/search?q=$query');
  }

  Uri allDocuments() {
    return Uri.parse('${_config.baseUrl}/thumbnails/all');
  }

  Uri thumbnail(String filename) {
    return Uri.parse('${_config.baseUrl}/thumbnail/$filename');
  }

  Uri document(String filename) {
    return Uri.parse('${_config.baseUrl}/document/$filename');
  }

  Uri upload() {
    return Uri.parse('${_config.baseUrl}/document/upload');
  }

  Uri notifications() {
    return Uri.parse(_config.websocketUrl);
  }
}
