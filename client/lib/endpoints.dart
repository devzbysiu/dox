import 'package:dox/config.dart';

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
}
