import 'package:dox/models/docs_state.dart';
import 'package:dox/models/document.dart';
import 'package:dox/utilities/config.dart';
import 'package:dox/widgets/search_input.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:provider/provider.dart';

Future<MultiProvider> wrap({required Widget widget, DocsState? docsState}) async {
  final docsSt = docsState ?? DocsStateMock();
  return MultiProvider(
    providers: [
      ChangeNotifierProvider<DocsState>(create: (_) => docsSt),
    ],
    child: MaterialApp(home: widget),
  );
}

class ConfigMock implements Config {
  @override
  String get baseUrl => 'http://192.168.16.247:8000';
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
