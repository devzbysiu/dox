import 'package:dox/utilities/config.dart';
import 'package:get_it/get_it.dart';

final getIt = GetIt.instance;

class Urls {
  late final Config _config;

  Urls({Config? config}) {
    _config = config ?? getIt.get<Config>();
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
