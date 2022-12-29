import 'package:dox/widgets/search_input.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../utils.dart';

void main() {
  testWidgets('SearchInput has a hint message', (tester) async {
    // given
    final docsState = DocsStateMock();
    const searchInput = SearchInput();

    // when
    await tester.pumpWidget(await withDocsState(searchInput, docsState));

    // then
    expect(searchInput.hintText(tester), equals('Search'));
  });

  testWidgets('SearchInput has a clear button', (tester) async {
    // given
    final docsState = DocsStateMock();
    const searchInput = SearchInput();

    // when
    await tester.pumpWidget(await withDocsState(searchInput, docsState));

    // then
    expect(find.byType(IconButton), findsOneWidget);
    expect(searchInput.icon(tester), equals(Icons.clear));
  });

  testWidgets('reset() is called on state when clear pressed', (tester) async {
    // given
    final docsState = DocsStateMock();
    const searchInput = SearchInput();
    await tester.pumpWidget(await withDocsState(searchInput, docsState));

    // when
    await tester.tap(find.byType(IconButton));

    // then
    expect(docsState.wasResetCalled, isTrue);
  });

  testWidgets('After tap on clear button, SearchInput clears', (tester) async {
    // given
    final docsState = DocsStateMock();
    const searchInput = SearchInput();
    await tester.pumpWidget(await withDocsState(searchInput, docsState));
    await tester.enterText(find.byType(TextField), 'Search phrase');
    expect(find.text('Search phrase'), findsOneWidget);

    // when
    await tester.tap(find.byType(IconButton));

    // then
    expect(find.text('Search phrase'), findsNothing);
  });

  testWidgets('onQueryChanged is called after changing input', (tester) async {
    // given
    final docsState = DocsStateMock();
    const searchInput = SearchInput();
    await tester.pumpWidget(await withDocsState(searchInput, docsState));
    expect(docsState.wasOnQueryChangedCalled, isFalse);

    // when
    await tester.enterText(find.byType(TextField), 'Search phrase');

    // then
    expect(docsState.wasOnQueryChangedCalled, isTrue);
  });
}
