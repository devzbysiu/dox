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

  testWidgets('When tapped, it unveils two more buttons', (tester) async {
    // given
    final docsServiceMock = DocsServiceMock();
    final scanServiceMock = ScanServiceMock();
    final addButton = AddButton(
      docsService: docsServiceMock,
      scanService: scanServiceMock,
    );
    await tester.pumpWidget(await wrap(widget: addButton));
    expect(find.byType(Icon), findsOneWidget);

    // when
    await tester.tap(find.byType(Icon));
    await tester.pump();

    // then
    expect(find.byType(Icon), findsNWidgets(3));
  });

  testWidgets('When tapped, I can find Pick PDF button', (tester) async {
    // given
    final docsServiceMock = DocsServiceMock();
    final scanServiceMock = ScanServiceMock();
    final addButton = AddButton(
      docsService: docsServiceMock,
      scanService: scanServiceMock,
    );
    await tester.pumpWidget(await wrap(widget: addButton));
    expect(find.text('Pick PDF'), findsNothing);

    // when
    await tester.tap(find.byType(Icon));
    await tester.pump();

    // then
    expect(find.text('Pick PDF'), findsOneWidget);
  });

  testWidgets('When tapped, I can find Scan document button', (tester) async {
    // given
    final docsServiceMock = DocsServiceMock();
    final scanServiceMock = ScanServiceMock();
    final addButton = AddButton(
      docsService: docsServiceMock,
      scanService: scanServiceMock,
    );
    await tester.pumpWidget(await wrap(widget: addButton));
    expect(find.text('Scan document'), findsNothing);

    // when
    await tester.tap(find.byType(Icon));
    await tester.pump();

    // then
    expect(find.text('Scan document'), findsOneWidget);
  });
}
