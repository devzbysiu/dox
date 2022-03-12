import 'dart:convert';
import 'dart:io';

import 'package:dox/models/document.dart';
import 'package:dox/utilities/events_stream.dart';
import 'package:dox/utilities/exceptions.dart';
import 'package:dox/utilities/urls.dart';
import 'package:flutter/foundation.dart';
import 'package:http/http.dart' as http;
import 'package:web_socket_channel/io.dart';

const filename = 'filename';
const thumbnail = 'thumbnail';
const fileUrl = 'fileUrl';
const thumbnailUrl = 'thumbnailUrl';

typedef VoidFunction = void Function()?;

class DocsService {
  late final Urls _urls;

  late final Stream _stream;

  static DocsService? _instance;

  static init(Urls urls, EventsStream stream) {
    _instance ??= DocsService._(urls, stream);
  }

  DocsService._(Urls urls, EventsStream stream) {
    _urls = urls;
    _stream = stream.stream; // TODO: Improve this repetition
  }

  factory DocsService() {
    if (_instance == null) throw ApiNotInitializedException();
    return _instance!;
  }

  Future<List<Document>> fetchAllFiles() async {
    return await _fetchDocs(_urls.allDocuments());
  }

  // TODO: think about pagination (or something similar)
  Future<List<Document>> _fetchDocs(Uri endpoint) async {
    final response = await http.get(endpoint);
    final body = json.decode(utf8.decode(response.bodyBytes));
    var entries = body['entries'] as List;
    return _toDocuments(_extendWithUrls(entries));
  }

  List<dynamic> _extendWithUrls(List<dynamic> entries) {
    return entries.map((e) {
      e[fileUrl] = _toDocUrl(e[filename]);
      e[thumbnailUrl] = _toThumbnailUrl(e[thumbnail]);
      return e;
    }).toList();
  }

  List<Document> _toDocuments(List<dynamic> entries) {
    return entries
        .map((e) => Document(e[fileUrl], e[thumbnailUrl]))
        .toSet()
        .toList();
  }

  Future<List<Document>> searchDocs(String query) async {
    return _fetchDocs(_urls.search(query));
  }

  Future<http.Response> uploadDoc(File file) async {
    final jsonBody = await compute(toJson, {'file': file});
    return http.post(_urls.upload(), body: jsonBody, headers: {
      'Content-Type': 'application/json',
      'Accept': 'application/json',
    });
  }

  Uri _toThumbnailUrl(String filename) {
    return _urls.thumbnail(filename);
  }

  Uri _toDocUrl(String filename) {
    return _urls.document(filename);
  }

  void onNewDoc(Function onNewDoc) {
    _stream.listen((data) {
      if (data == "new-doc") {
        onNewDoc();
      }
    });
  }
}

class _Doc {
  late final String _filename;

  late final String _body;

  _Doc(File file) {
    _filename = _name(file);
    _body = base64Encode(file.readAsBytesSync());
  }

  String _name(File file) {
    return file.path.split('/').last;
  }

  Map<String, dynamic> toJson() {
    return {'filename': _filename, 'body': _body};
  }
}

Future<String> toJson(Map<String, dynamic> data) async {
  final file = data['file'];
  return jsonEncode(_Doc(file));
}
