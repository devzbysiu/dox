import 'package:dox/utilities/config.dart';

class Urls {
  late final Config _config;

  Urls(config) {
    _config = config;
  }

  Uri search(String query) {
    return Uri.parse('${_config.baseUrl}/search?q=$query');
  }

  Uri allDocuments() {
    return Uri.parse('${_config.baseUrl}/documents/all');
  }

  Uri document(String filename) {
    return Uri.parse('${_config.baseUrl}/document/$filename');
  }

  Uri upload(String filename) {
    return Uri.parse('${_config.baseUrl}/document/upload?name=$filename');
  }
}
