import 'dart:io';

import 'package:dox/widgets/add_button.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../utils.dart';

void main() {
  final anyFile = File('/some/path');

  testWidgets('AddButton is initially closed', (tester) async {
    // given
    final docsServiceSpy = DocsServiceSpy();
    final scanServiceSpy = ScanServiceSpy(scannedFile: anyFile);
    final addButton = AddButton(
      docsService: docsServiceSpy,
      scanService: scanServiceSpy,
    );

    // when
    await tester.pumpWidget(await wrap(widget: addButton));

    // then
    expect(addButton.icon(tester), equals(Icons.add));
  });

  testWidgets('When tapped, it unveils two more buttons', (tester) async {
    // given
    final docsServiceSpy = DocsServiceSpy();
    final scanServiceSpy = ScanServiceSpy(scannedFile: anyFile);
    final addButton = AddButton(
      docsService: docsServiceSpy,
      scanService: scanServiceSpy,
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
    final docsServiceSpy = DocsServiceSpy();
    final scanServiceSpy = ScanServiceSpy(scannedFile: anyFile);
    final addButton = AddButton(
      docsService: docsServiceSpy,
      scanService: scanServiceSpy,
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
    final docsServiceSpy = DocsServiceSpy();
    final scanServiceSpy = ScanServiceSpy(scannedFile: anyFile);
    final addButton = AddButton(
      docsService: docsServiceSpy,
      scanService: scanServiceSpy,
    );
    await tester.pumpWidget(await wrap(widget: addButton));
    expect(find.text('Scan document'), findsNothing);

    // when
    await tester.tap(find.byType(Icon));
    await tester.pump();

    // then
    expect(find.text('Scan document'), findsOneWidget);
  });

  testWidgets('Tap on Pick PDF button triggers PDF picker', (tester) async {
    // given
    final docsServiceSpy = DocsServiceSpy();
    final scanServiceSpy = ScanServiceSpy(scannedFile: anyFile);
    final addButton = AddButton(
      docsService: docsServiceSpy,
      scanService: scanServiceSpy,
    );
    await tester.pumpWidget(await wrap(widget: addButton));
    await tester.tap(find.byType(Icon));
    await tester.pump();
    expect(scanServiceSpy.wasPickPdfCalled, isFalse);

    // when
    await tester.tap(find.text('Pick PDF'));
    await tester.pump();

    // then
    expect(scanServiceSpy.wasPickPdfCalled, isTrue);
  });

  testWidgets('Tap on Scan doc button triggers doc scanner', (tester) async {
    // given
    final docsServiceSpy = DocsServiceSpy();
    final scanServiceSpy = ScanServiceSpy(scannedFile: anyFile);
    final addButton = AddButton(
      docsService: docsServiceSpy,
      scanService: scanServiceSpy,
    );
    await tester.pumpWidget(await wrap(widget: addButton));
    await tester.tap(find.byType(Icon));
    await tester.pump();
    expect(scanServiceSpy.wasScanImageCalled, isFalse);

    // when
    await tester.tap(find.text('Scan document'));
    await tester.pump();

    // then
    expect(scanServiceSpy.wasScanImageCalled, isTrue);
  });

  testWidgets('Tap on Pick PDF button, PDF is sent', (tester) async {
    // given
    final docsServiceSpy = DocsServiceSpy();
    final scanServiceSpy = ScanServiceSpy(scannedFile: anyFile);
    final addButton = AddButton(
      docsService: docsServiceSpy,
      scanService: scanServiceSpy,
    );
    await tester.pumpWidget(await wrap(widget: addButton));
    await tester.tap(find.byType(Icon));
    await tester.pump();
    expect(docsServiceSpy.wasUploadDocCalled, isFalse);

    // when
    await tester.tap(find.text('Pick PDF'));
    await tester.pump();

    // then
    expect(docsServiceSpy.wasUploadDocCalled, isTrue);
  });

  testWidgets('Tap on Scan doc button, doc is sent', (tester) async {
    // given
    final docsServiceSpy = DocsServiceSpy();
    final scanServiceSpy = ScanServiceSpy(scannedFile: anyFile);
    final addButton = AddButton(
      docsService: docsServiceSpy,
      scanService: scanServiceSpy,
    );
    await tester.pumpWidget(await wrap(widget: addButton));
    await tester.tap(find.byType(Icon));
    await tester.pump();
    expect(docsServiceSpy.wasUploadDocCalled, isFalse);

    // when
    await tester.tap(find.text('Scan document'));
    await tester.pump();

    // then
    expect(docsServiceSpy.wasUploadDocCalled, isTrue);
  });

  testWidgets('PDF is not send when selected file is null', (tester) async {
    // given
    final docsServiceSpy = DocsServiceSpy();
    final scanServiceSpy = ScanServiceSpy(scannedFile: null);
    final addButton = AddButton(
      docsService: docsServiceSpy,
      scanService: scanServiceSpy,
    );
    await tester.pumpWidget(await wrap(widget: addButton));
    await tester.tap(find.byType(Icon));
    await tester.pump();
    expect(docsServiceSpy.wasUploadDocCalled, isFalse);

    // when
    await tester.tap(find.text('Pick PDF'));
    await tester.pump();

    // then
    expect(docsServiceSpy.wasUploadDocCalled, isFalse);
  });

  testWidgets('Doc is not send when selected file is null', (tester) async {
    // given
    final docsServiceSpy = DocsServiceSpy();
    final scanServiceSpy = ScanServiceSpy(scannedFile: null);
    final addButton = AddButton(
      docsService: docsServiceSpy,
      scanService: scanServiceSpy,
    );
    await tester.pumpWidget(await wrap(widget: addButton));
    await tester.tap(find.byType(Icon));
    await tester.pump();
    expect(docsServiceSpy.wasUploadDocCalled, isFalse);

    // when
    await tester.tap(find.text('Scan document'));
    await tester.pump();

    // then
    expect(docsServiceSpy.wasUploadDocCalled, isFalse);
  });

  // NOTE: When sending fails, the toast is shown, but currently, I did not find
  // any way to test if toast is visible.
  testWidgets('When sending PDF failed, nothing happens', (tester) async {
    // given
    final docsServiceSpy = DocsServiceSpy(uploadStatusCode: 500);
    final scanServiceSpy = ScanServiceSpy(scannedFile: anyFile);
    final addButton = AddButton(
      docsService: docsServiceSpy,
      scanService: scanServiceSpy,
    );
    await tester.pumpWidget(await wrap(widget: addButton));
    await tester.tap(find.byType(Icon));
    await tester.pump();

    // when
    await tester.tap(find.text('Pick PDF'));
    await tester.pumpAndSettle(const Duration(seconds: 2));

    // then
    // no errors
  });

  // NOTE: When sending fails, the toast is shown, but currently, I did not find
  // any way to test if toast is visible.
  testWidgets('When sending doc failed, nothing happens', (tester) async {
    // given
    final docsServiceSpy = DocsServiceSpy(uploadStatusCode: 500);
    final scanServiceSpy = ScanServiceSpy(scannedFile: anyFile);
    final addButton = AddButton(
      docsService: docsServiceSpy,
      scanService: scanServiceSpy,
    );
    await tester.pumpWidget(await wrap(widget: addButton));
    await tester.tap(find.byType(Icon));
    await tester.pump();

    // when
    await tester.tap(find.text('Scan document'));
    await tester.pumpAndSettle(const Duration(seconds: 2));

    // then
    // no errors
  });
}
