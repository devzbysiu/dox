import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:http/http.dart' as http;

import 'document.dart';

class SearchModel extends ChangeNotifier {
  bool _isLoading = false;

  bool get isLoading => _isLoading;

  List<Document> _suggestions = history;

  List<Document> get suggestions => _suggestions;

  String _query = '';

  String get query => _query;

  void onQueryChanged(String query) async {
    if (query == _query) return;

    _query = query;
    _isLoading = true;
    notifyListeners();

    if (query.isEmpty) {
      _suggestions = history;
    } else {
      final response =
          await http.get(Uri.parse('http://10.0.2.2:8000/search?q=$query'));

      final body = json.decode(utf8.decode(response.bodyBytes));
      final entries = body['entries'] as List;

      _suggestions = entries.map((e) => Document.fromJson(e)).toSet().toList();
    }

    _isLoading = false;
    notifyListeners();
  }

  void clear() {
    _suggestions = history;
    notifyListeners();
  }
}

const List<Document> history = [
  Document(filename: 'doc1.png'),
  Document(filename: 'doc1.png'),
  Document(filename: 'doc1.png'),
  Document(filename: 'doc1.png'),
];
