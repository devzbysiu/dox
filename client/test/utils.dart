import 'dart:io';

import 'package:dox/models/docs_state.dart';
import 'package:dox/models/document.dart';
import 'package:dox/services/scan_service.dart';
import 'package:dox/services/docs_service.dart';
import 'package:dox/services/sign_in_service.dart';
import 'package:dox/utilities/config.dart';
import 'package:dox/widgets/add_button.dart';
import 'package:dox/widgets/search_input.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/src/response.dart';
import 'package:provider/provider.dart';

Future<MultiProvider> wrap(
    {required Widget widget, DocsState? docsState}) async {
  final docsSt = docsState ?? DocsStateSpy();
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

class DocsStateSpy extends ChangeNotifier implements DocsState {
  DocsStateSpy({
    this.loading = false,
    this.docs = const [],
    this.resetCalled = false,
    this.onQueryChangedCalled = false,
  });

  bool loading;

  List<Document> docs;

  bool resetCalled;

  bool onQueryChangedCalled;

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

extension AddButtonExt on AddButton {
  IconData icon(WidgetTester tester) {
    final Icon icon = tester.firstWidget(find.byType(Icon));
    return icon.icon!;
  }
}

class DocsServiceSpy implements DocsService {
  DocsServiceSpy({
    this.fetchAllFilesCalled = false,
    this.searchDocsCalled = false,
    this.uploadDocCalled = false,
    this.uploadStatusCode = 201,
  });

  bool fetchAllFilesCalled;

  bool get wasFetchAllFilesCalled => fetchAllFilesCalled;

  bool searchDocsCalled;

  bool get wasSearchDocsCalled => searchDocsCalled;

  bool uploadDocCalled;

  bool get wasUploadDocCalled => uploadDocCalled;

  int uploadStatusCode;

  @override
  Future<List<Document>> fetchAllFiles() {
    fetchAllFilesCalled = true;
    return Future.value(List.empty());
  }

  @override
  Future<List<Document>> searchDocs(String query) {
    searchDocsCalled = true;
    return Future.value(List.empty());
  }

  @override
  Future<Response> uploadDoc(File file) {
    uploadDocCalled = true;
    return Future.value(Response('', uploadStatusCode));
  }
}

class FailingDocsServiceSpy implements DocsService {
  FailingDocsServiceSpy({
    this.fetchAllFilesCalled = false,
    this.searchDocsCalled = false,
    this.uploadDocCalled = false,
  });

  bool fetchAllFilesCalled;

  bool get wasFetchAllFilesCalled => fetchAllFilesCalled;

  bool searchDocsCalled;

  bool get wasSearchDocsCalled => searchDocsCalled;

  bool uploadDocCalled;

  bool get wasUploadDocCalled => uploadDocCalled;

  @override
  Future<List<Document>> fetchAllFiles() {
    fetchAllFilesCalled = true;
    throw Exception('Failed to fetch all files');
  }

  @override
  Future<List<Document>> searchDocs(String query) {
    searchDocsCalled = true;
    throw Exception('Failed to search docs');
  }

  @override
  Future<Response> uploadDoc(File file) {
    uploadDocCalled = true;
    throw Exception('Failed to upload doc');
  }
}

class ScanServiceSpy implements ScanService {
  ScanServiceSpy({
    this.pickPdfCalled = false,
    this.scanImageCalled = false,
    required this.scannedFile,
  });

  bool pickPdfCalled;

  bool get wasPickPdfCalled => pickPdfCalled;

  bool scanImageCalled;

  bool get wasScanImageCalled => scanImageCalled;

  File? scannedFile;

  @override
  Future<File?> pickPdf() {
    pickPdfCalled = true;
    return Future.value(scannedFile);
  }

  @override
  Future<File?> scanImage(BuildContext context) {
    scanImageCalled = true;
    return Future.value(scannedFile);
  }
}

class SignInServiceDummy implements SignInService {
  @override
  Map<String, String> get authHeaders => <String, String>{};

  @override
  Future<void> signIn() {
    throw UnimplementedError();
  }
}
