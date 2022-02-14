import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

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
    http.post(_urls.upload(),
        body: jsonEncode(FileUpload(file)),
        headers: {
          'Content-Type': 'application/json',
          'Accept': 'application/json',
        });
  }

  // Future<void> uploadDoc(File file) async {
  //   var req = http.MultipartRequest('POST', _urls.upload(_name(file)));
  //   req.files.add(_multipartFile(file));
  //   await req.send();
  // }

  http.MultipartFile _multipartFile(File file) {
    return http.MultipartFile('document', _stream(file), _length(file),
        filename: _name(file));
  }

  Stream<Uint8List> _stream(File file) {
    return file.readAsBytes().asStream();
  }

  int _length(File file) {
    return file.lengthSync();
  }

  String _name(File file) {
    return file.path.split('/').last;
  }

  Uri toDocUrl(String filename) {
    return _urls.document(filename);
  }
}

class FileUpload {
  late final String _name;

  late final String _body;

  FileUpload(File file) {
    _name = file.path.split('/').last;
    _body = base64Encode(file.readAsBytesSync());
  }

  Map<String, dynamic> toJson() {
    return {
      'name': _name,
      'body': _body
    };
  }
}
