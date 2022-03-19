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
    expect(searchInput.clearButton(tester), isNotNull);
    expect(searchInput.icon(tester), equals(Icons.clear));
  });
}