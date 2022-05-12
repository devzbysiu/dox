import 'package:dox/models/docs_state.dart';
import 'package:dox/models/document.dart';
import 'package:dox/services/docs_service.dart';
import 'package:dox/utilities/config.dart';
import 'package:dox/utilities/connection.dart';
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
  Connection? conn,
  DocsService? docs,
  DocsState? docsSt,
}) {
  final config = cfg ?? ConfigMock();
  final urlsProvider = urls ?? Urls(config: config);
  final docsService = docs ?? DocsService(urls: urlsProvider);
  DocsState docsState(_) => docsSt ?? DocsStateImpl(docsService: docsService);
  Connection connState(_) => conn ?? ConnectionImpl(urlsProvider: urlsProvider);

  return MultiProvider(
    providers: [
      ChangeNotifierProvider<DocsState>(create: docsState),
      ChangeNotifierProvider<Connection>(create: connState),
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

class DocsStateMock extends ChangeNotifier implements DocsState {
  bool loading;

  List<Document> docs;

  bool resetCalled;

  bool onQueryChangedCalled;

  DocsStateMock({
    this.loading = false,
    this.docs = const [],
    this.resetCalled = false,
    this.onQueryChangedCalled = false,
  });

  @override
  bool get isLoading => loading;

  @override
  List<Document> get suggestions => docs;

  bool get wasResetCalled => resetCalled;

  bool get wasOnQueryChangedCalled => onQueryChangedCalled;

  @override
  Future<void> onQueryChanged(String query) async {
    onQueryChangedCalled = true;
  }

  @override
  Future<void> refresh() {
    return Future.delayed(const Duration(microseconds: 250));
  }

  @override
  Future<void> reset() async {
    resetCalled = true;
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

  IconData icon(WidgetTester tester) {
    final IconButton button = tester.firstWidget(find.byType(IconButton));
    final icon = button.icon as Icon;
    return icon.icon!;
  }
}

List<Color> connectedColor() {
  return [Colors.green[300]!, Colors.yellow[400]!];
}

List<Color> disconnectedColor() {
  return [Colors.blueGrey, Colors.blueGrey];
}

class ConnectionMock extends ChangeNotifier implements Connection {
  @override
  void disconnect() {
    // nothing to do
  }

  @override
  void onConnected(Function fun) {
    onConnectedFun = fun;
  }

  @override
  void onDisconnected(Function fun) {
    onDisconnectedFun = fun;
  }

  @override
  void onNewDoc(Function fun) {
    // nothing to do
  }

  @override
  void reconnect() {
    // nothing to do
  }

  void forceConnected() {
    onConnectedFun();
  }

  void forceDisconnected() {
    onDisconnectedFun();
  }

  late Function onConnectedFun;

  late Function onDisconnectedFun;
}