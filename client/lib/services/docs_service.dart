import 'dart:convert';
import 'dart:io';

import 'package:dox/models/document.dart';
import 'package:dox/utilities/http.dart';
import 'package:dox/utilities/log.dart';
import 'package:dox/utilities/service_locator.dart';
import 'package:dox/utilities/urls.dart';
import 'package:flutter/foundation.dart';
import 'package:http/http.dart' show Response;

const filename = 'filename';
const thumbnail = 'thumbnail';
const fileUrl = 'fileUrl';
const thumbnailUrl = 'thumbnailUrl';

class DocsService with Log {
  DocsService({
    Urls? urls,
    AuthenticatedClient? http,
  }) {
    log.fine('initializing DocsService');
    _urls = urls ?? getIt<Urls>();
    _http = http ?? getIt<AuthenticatedClient>();
  }

  late final Urls _urls;

  late final AuthenticatedClient _http;

  Future<List<Document>> fetchAllFiles() async {
    log.fine('fetching all files');
    return await _fetchDocs(_urls.allDocuments());
  }

  // TODO: think about pagination (or something similar)
  Future<List<Document>> _fetchDocs(Uri endpoint) async {
    log.fine('calling endpoint: "$endpoint"');
    final response = await _http.get(endpoint);
    log.fine('got response code: ${response.statusCode}');
    log.fine('decoding to json');
    final body = json.decode(utf8.decode(response.bodyBytes));
    var entries = body['entries'] as List;
    log.fine('mapping to documents');
    return _toDocuments(_extendWithUrls(entries));
  }

  List<dynamic> _extendWithUrls(List<dynamic> entries) {
    log.fine('extending with URLs');
    return entries.map((e) {
      e[fileUrl] = _toDocUrl(e[filename]);
      e[thumbnailUrl] = _toThumbnailUrl(e[thumbnail]);
      return e;
    }).toList();
  }

  List<Document> _toDocuments(List<dynamic> entries) {
    log.fine('creating documents from entries');
    return entries
        .map((e) => Document(e[fileUrl], e[thumbnailUrl]))
        .toSet()
        .toList();
  }

  Future<List<Document>> searchDocs(String query) async {
    log.fine('searching docs using query: "$query"');
    return _fetchDocs(_urls.search(query));
  }

  Future<Response> uploadDoc(File file) async {
    log.fine('uploading doc using file: "${file.path}"');
    final jsonBody = await compute(toJson, {'file': file});
    return _http.post(_urls.upload(), body: jsonBody, headers: {
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
}

class _Doc {
  _Doc(File file) {
    _filename = _name(file);
    _body = base64Encode(file.readAsBytesSync());
  }

  late final String _filename;

  late final String _body;

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
