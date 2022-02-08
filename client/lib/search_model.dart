import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:http/http.dart' as http;

import 'document.dart';

class SearchModel extends ChangeNotifier {
  bool _isLoading = false;

  bool get isLoading => _isLoading;

  // TODO: this cannot be empty on start
  List<Document> _suggestions = List.empty();

  List<Document> get suggestions => _suggestions;

  String _query = '';

  String get query => _query;

  void onQueryChanged(String query) async {
    if (query == _query) return;

    _query = query;
    _isLoading = true;
    notifyListeners();

    if (query.isEmpty) {
      _suggestions = await allDocuments();
    } else {
      // TODO: move URLs somewhere
      final response =
          await http.get(Uri.parse('http://10.0.2.2:8000/search?q=$query'));

      final body = json.decode(utf8.decode(response.bodyBytes));
      final entries = body['entries'] as List;

      _suggestions = entries.map((e) => Document.fromJson(e)).toSet().toList();
    }

    _isLoading = false;
    notifyListeners();
  }

  // TODO: think about pagination (or something similar)
  Future<List<Document>> allDocuments() async {
    // TODO: DRY
    final response =
        await http.get(Uri.parse('http://10.0.2.2:8000/documents/all'));
    final body = json.decode(utf8.decode(response.bodyBytes));
    final entries = body['entries'] as List;
    return entries.map((e) => Document.fromJson(e)).toSet().toList();
  }

  void clear() async {
    _suggestions = await allDocuments();
    notifyListeners();
  }
}
