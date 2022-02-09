import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:http/http.dart' as http;

import 'config.dart';
import 'document.dart';
import 'endpoints.dart';

class SearchModel extends ChangeNotifier {
  late bool _isLoading;

  late List<Document> _suggestions;

  late String _query;

  late final Urls _urls;

  SearchModel(Config config) {
    _isLoading = false;
    _suggestions = List.empty();
    _urls = Urls(config);
    _query = '';
    fetchDocs(_urls.allDocuments()).then((value) {
      _suggestions = value;
      notifyListeners();
    });
  }

  void onQueryChanged(String query) async {
    if (query == _query) return;

    _query = query;
    _isLoading = true;
    notifyListeners();

    final uri = query.isEmpty ? _urls.allDocuments() : _urls.search(query);
    _suggestions = await fetchDocs(uri);

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
    _suggestions = await fetchDocs(_urls.allDocuments());
    notifyListeners();
  }

  bool get isLoading => _isLoading;

  List<Document> get suggestions => _suggestions;
}
