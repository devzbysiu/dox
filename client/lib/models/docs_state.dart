import 'dart:io';

import 'package:dox/models/document.dart';
import 'package:dox/services/docs_service.dart';
import 'package:flutter/material.dart';
import 'package:get_it/get_it.dart';

final getIt = GetIt.instance;

class DocsState extends ChangeNotifier {
  bool _isLoading = false;

  List<Document> _suggestions = List.empty();

  String _query = '';

  late final DocsService _docsService;

  DocsState({
    DocsService? docsService,
  }) {
    _docsService = docsService ?? getIt.get<DocsService>();
    _docsService.onNewDoc(refresh);
    _docsService.fetchAllFiles().then((value) {
      _suggestions = value;
      notifyListeners();
    });
  }

  Future<void> refresh() async {
    reset();
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
        ? await _docsService.fetchAllFiles()
        : await _docsService.searchDocs(query);
  }

  Future<void> reset() async {
    _suggestions = await _docsService.fetchAllFiles();
    notifyListeners();
  }

  Future<bool> newDoc(File doc) async {
    final resp = await _docsService.uploadDoc(doc);
    if (resp.statusCode != 201) {
      return false;
    }
    return true;
  }

  bool get isLoading => _isLoading;

  List<Document> get suggestions => _suggestions;
}
