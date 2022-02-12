import 'dart:convert';

import 'package:dox/models/document.dart';
import 'package:dox/utilities/dox_service.dart';
import 'package:flutter/material.dart';
import 'package:http/http.dart' as http;

class SearchModel extends ChangeNotifier {
  late bool _isLoading;

  late List<Document> _suggestions;

  late String _query;

  late final DoxService _dox;

  SearchModel(DoxService dox) {
    _isLoading = false;
    _suggestions = List.empty();
    _dox = dox;
    _query = '';
    _dox.fetchAllFiles().then((value) {
      _suggestions = value;
      notifyListeners();
    });
  }

  void onQueryChanged(String query) async {
    if (query == _query) return;

    _query = query;
    _isLoading = true;
    notifyListeners();

    _suggestions = query.isEmpty
        ? await _dox.fetchAllFiles()
        : await _dox.searchDocs(query);

    _isLoading = false;
    notifyListeners();
  }

  // TODO: think about pagination (or something similar)
  Future<List<Document>> fetchDocs(Uri endpoint) async {
    final response = await http.get(endpoint);
    final body = json.decode(utf8.decode(response.bodyBytes));
    final entries = body['entries'] as List;
    return entries.map((e) => Document.fromJson(e)).toSet().toList();
  }

  void clear() async {
    _suggestions = await _dox.fetchAllFiles();
    notifyListeners();
  }

  bool get isLoading => _isLoading;

  List<Uri> get docUrls {
    return _suggestions.map((doc) => _dox.toDocUrl(doc.filename)).toList();
  }
}
