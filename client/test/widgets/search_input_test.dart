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
    await tester.pumpWidget(wrapper(widget: searchInput, docsSt: docsState));

    // then
    expect(searchInput.hintText(tester), equals('Search'));
  });

  testWidgets('SearchInput has a clear button', (tester) async {
    // given
    final docsState = DocsStateMock();
    const searchInput = SearchInput();

    // when
    await tester.pumpWidget(wrapper(widget: searchInput, docsSt: docsState));

    // then
    expect(find.byType(IconButton), findsOneWidget);
    expect(searchInput.icon(tester), equals(Icons.clear));
  });

  testWidgets('reset() is called on state when clear pressed', (tester) async {
    // given
    final docsState = DocsStateMock();
    const searchInput = SearchInput();
    await tester.pumpWidget(wrapper(widget: searchInput, docsSt: docsState));

    // when
    await tester.tap(find.byType(IconButton));

    // then
    expect(docsState.wasResetCalled, isTrue);
  });

  testWidgets('After tap on clear button, SearchInput clears', (tester) async {
    // given
    final docsState = DocsStateMock();
    const searchInput = SearchInput();
    await tester.pumpWidget(wrapper(widget: searchInput, docsSt: docsState));
    await tester.enterText(find.byType(TextField), 'Search phrase');
    expect(find.text('Search phrase'), findsOneWidget);

    // when
    await tester.tap(find.byType(IconButton));
    await tester.pump();

    // then
    expect(find.text('Search phrase'), findsNothing);
  });
}