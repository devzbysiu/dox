import 'dart:io';

import 'package:dox/models/document.dart';
import 'package:dox/services/docs_service.dart';
import 'package:dox/utilities/log.dart';
import 'package:dox/utilities/service_locator.dart';
import 'package:flutter/material.dart';

class DocsState extends ChangeNotifier with Log {
  bool _isLoading = false;

  List<Document> _suggestions = List.empty();

  String _query = '';

  late final DocsService _docsService;

  DocsState({
    DocsService? docsService,
  }) {
    log.fine('initializing DocsState');
    _docsService = docsService ?? getIt<DocsService>();
    _docsService.onNewDoc(refresh);
    _docsService.fetchAllFiles().then((value) {
      log.fine('fetched all docs, notifying');
      _suggestions = value;
      notifyListeners();
    });
  }

  Future<void> refresh() async {
    log.fine('refreshing');
    reset();
  }

  void onQueryChanged(String query) async {
    if (query == _query) return;
    log.fine('new query: $query');

    _query = query;
    _isLoading = true;
    notifyListeners();

    _suggestions = await _giveSuggestions(query);

    _isLoading = false;
    notifyListeners();
  }

  Future<List<Document>> _giveSuggestions(String query) async {
    log.fine('getting suggestions');
    return query.isEmpty
        ? await _docsService.fetchAllFiles()
        : await _docsService.searchDocs(query);
  }

  Future<void> reset() async {
    log.fine('resetting to showing all docs');
    _suggestions = await _docsService.fetchAllFiles();
    notifyListeners();
  }

  Future<bool> newDoc(File doc) async {
    log.fine('adding new doc');
    final resp = await _docsService.uploadDoc(doc);
    if (resp.statusCode != 201) {
      log.warning('failed to send new doc: ${resp.statusCode} -> ${resp.body}');
      return false;
    }
    log.fine('new doc sent');
    return true;
  }

  bool get isLoading => _isLoading;

  List<Document> get suggestions => _suggestions;
}
