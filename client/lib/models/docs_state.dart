import 'package:dox/models/document.dart';
import 'package:dox/services/docs_service.dart';
import 'package:dox/utilities/log.dart';
import 'package:dox/utilities/service_locator.dart';
import 'package:flutter/material.dart';

abstract class DocsState extends ChangeNotifier {
  bool get isLoading;
  List<Document> get suggestions;
  Future<void> onQueryChanged(String query);
  Future<void> refresh();
  Future<void> reset();
}

class DocsStateImpl extends ChangeNotifier with Log implements DocsState {
  DocsStateImpl({
    DocsService? docsService,
  }) {
    log.fine('initializing DocsState');
    _docsService = docsService ?? getIt<DocsService>();
    _docsService.fetchAllFiles().then((value) {
      log.fine('fetched all docs, notifying');
      _suggestions = value;
      notifyListeners();
    });
  }

  bool _isLoading = false;

  List<Document> _suggestions = List.empty();

  String _query = '';

  late final DocsService _docsService;

  @override
  bool get isLoading => _isLoading;

  @override
  List<Document> get suggestions => _suggestions;

  @override
  Future<void> refresh() async {
    log.fine('refreshing');
    reset();
  }

  @override
  Future<void> onQueryChanged(String query) async {
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

  @override
  Future<void> reset() async {
    log.fine('resetting to showing all docs');
    _suggestions = await _docsService.fetchAllFiles();
    notifyListeners();
  }
}
