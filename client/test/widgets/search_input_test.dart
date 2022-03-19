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
    // just 'pump()' is not enough because Future.delayed in the mock
    await tester.pumpAndSettle();

    // then
    expect(docsState.wasResetCalled, isTrue);
  });
}