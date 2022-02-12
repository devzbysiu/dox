import 'package:dox/models/document.dart';
import 'package:dox/utilities/dox_service.dart';
import 'package:flutter/material.dart';

class SearchModel extends ChangeNotifier {
  bool _isLoading = false;

  List<Document> _suggestions = List.empty();

  String _query = '';

  late final DoxService _dox;

  SearchModel(DoxService dox) {
    _dox = dox;
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

    _suggestions = await _giveSuggestions(query);

    _isLoading = false;
    notifyListeners();
  }

  Future<List<Document>> _giveSuggestions(String query) async {
    return query.isEmpty
        ? await _dox.fetchAllFiles()
        : await _dox.searchDocs(query);
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
