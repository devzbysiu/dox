import 'dart:convert';
import 'dart:io';

import 'package:dox/models/document.dart';
import 'package:dox/utilities/urls.dart';
import 'package:http/http.dart' as http;

class DoxService {
  late final Urls _urls;

  DoxService(Urls urls) {
    _urls = urls;
  }

  Future<List<Document>> fetchAllFiles() async {
    return await _fetchDocs(_urls.allDocuments());
  }

  // TODO: think about pagination (or something similar)
  Future<List<Document>> _fetchDocs(Uri endpoint) async {
    final response = await http.get(endpoint);
    final body = json.decode(utf8.decode(response.bodyBytes));
    final entries = body['entries'] as List;
    return entries.map((e) => Document.fromJson(e)).toSet().toList();
  }

  Future<List<Document>> searchDocs(String query) async {
    return _fetchDocs(_urls.search(query));
  }

  Future<void> uploadDoc(File file) async {
    http.post(_urls.upload(), body: jsonEncode(_Document(file)), headers: {
      'Content-Type': 'application/json',
      'Accept': 'application/json',
    });
  }

  Uri toDocUrl(String filename) {
    return _urls.document(filename);
  }
}

class _Document {
  late final String _filename;

  late final String _body;

  _Document(File file) {
    _filename = _name(file);
    // TODO: do I need to move it to isolate?
    _body = base64Encode(file.readAsBytesSync());
  }

  String _name(File file) {
    return file.path.split('/').last;
  }

  Map<String, dynamic> toJson() {
    return {'filename': _filename, 'body': _body};
  }
}
