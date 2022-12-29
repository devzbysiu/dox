import 'package:dox/widgets/add_button.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../utils.dart';

void main() {
  testWidgets('AddButton is initially closed', (tester) async {
    // given
    final docsServiceMock = DocsServiceMock();
    final scanServiceMock = ScanServiceMock();
    final addButton = AddButton(
      docsService: docsServiceMock,
      scanService: scanServiceMock,
    );

    // when
    await tester.pumpWidget(await wrap(widget: addButton));

    // then
    expect(addButton.icon(tester), equals(Icons.add));
  });
}
