import 'package:dox/models/connection_state.dart';
import 'package:dox/models/docs_state.dart';
import 'package:dox/models/document.dart';
import 'package:dox/services/connection_service.dart';
import 'package:dox/services/docs_service.dart';
import 'package:dox/utilities/config.dart';
import 'package:dox/utilities/events_stream.dart';
import 'package:dox/utilities/urls.dart';
import 'package:dox/widgets/search_input.dart';
import 'package:dox/widgets/status_dot.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:provider/provider.dart';

MultiProvider wrapper({
  required widget,
  Config? cfg,
  Urls? urls,
  Events? ev,
  DocsService? docs,
  DocsState? docsSt,
  ConnService? conn,
  ConnState? connSt,
}) {
  final config = cfg ?? ConfigMock();
  final urlsProvider = urls ?? Urls(config: config);
  final events = ev ?? Events(urlsProvider: urlsProvider);
  final docsService = docs ?? DocsService(urls: urlsProvider, ev: events);
  final connService = conn ?? ConnService(ev: events);
  DocsState docsState(_) => docsSt ?? DocsStateImpl(docsService: docsService);
  ConnState connState(_) => connSt ?? ConnStateImpl(connService: connService);

  return MultiProvider(
    providers: [
      ChangeNotifierProvider<DocsState>(create: docsState),
      ChangeNotifierProvider<ConnState>(create: connState),
    ],
    child: MaterialApp(home: widget),
  );
}

class ConfigMock implements Config {
  @override
  String get baseUrl => 'http://192.168.16.247:8000';

  @override
  String get websocketUrl => 'ws://192.168.16.247:8001';
}

class ConnStateMock extends ChangeNotifier implements ConnState {
  bool _isConnected = false;

  @override
  bool get isConnected => _isConnected;

  set isConnected(val) {
    _isConnected = val;
    notifyListeners();
  }
}

class DocsStateMock extends ChangeNotifier implements DocsState {
  bool loading;

  List<Document> docs;

  DocsStateMock({this.loading = false, this.docs = const []});

  @override
  bool get isLoading => loading;

  @override
  List<Document> get suggestions => docs;

  @override
  Future<void> onQueryChanged(String query) {
    return Future.delayed(const Duration(microseconds: 250));
  }

  @override
  Future<void> refresh() {
    return Future.delayed(const Duration(microseconds: 250));
  }
}

extension StatusDotExt on StatusDot {
  List<Color> color(WidgetTester tester) {
    final Container container = tester.firstWidget(find.byType(Container));
    final boxDecoration = container.decoration as BoxDecoration;
    final gradient = boxDecoration.gradient as LinearGradient;
    return gradient.colors;
  }
}

extension SearchInputExt on SearchInput {
  String hintText(WidgetTester tester) {
    final TextField input = tester.firstWidget(find.byType(TextField));
    final decoration = input.decoration as InputDecoration;
    return decoration.hintText!;
  }
}
