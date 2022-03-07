import 'dart:io';

import 'package:dox/models/document.dart';
import 'package:dox/utilities/api.dart';
import 'package:flutter/material.dart';

class SearchModel extends ChangeNotifier {
  bool _isLoading = false;

  List<Document> _suggestions = List.empty();

  String _query = '';

  late final Api _api;

  SearchModel(Api api) {
    _api = api;
    _api.fetchAllFiles().then((value) {
      _suggestions = value;
      notifyListeners();
    });
  }

  void onQueryChanged(String query) async {
    if (query == _query) return;

    _query = query;
    _isLoading = true;
    notifyListeners();

    _suggestions = await _giveSuggestions(query);

    _isLoading = false;
    notifyListeners();
  }

  Future<List<Document>> _giveSuggestions(String query) async {
    return query.isEmpty
        ? await _api.fetchAllFiles()
        : await _api.searchDocs(query);
  }

  Future<void> clear() async {
    _suggestions = await _api.fetchAllFiles();
    notifyListeners();
  }

  Future<void> refresh() async {
    clear();
  }

  Future<bool> newDoc(File doc) async {
    final resp = await _api.uploadDoc(doc);
    if (resp.statusCode != 201) {
      return false;
    }
    return true;
  }

  bool get isLoading => _isLoading;

  List<Document> get suggestions => _suggestions;
}
