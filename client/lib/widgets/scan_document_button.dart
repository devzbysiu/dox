import 'dart:io';
import 'dart:typed_data';

import 'package:document_scanner_flutter/document_scanner_flutter.dart';
import 'package:dox/utilities/urls.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:http/http.dart' as http;

class ScanDocumentButton extends StatelessWidget {
  late final Urls _urls;

  ScanDocumentButton(Urls urls, {Key? key}) : super(key: key) {
    _urls = urls;
  }

  @override
  Widget build(BuildContext context) {
    return FloatingActionButton(
      onPressed: () => _scanAndSendDocument(context),
      backgroundColor: Colors.orange,
      child: const Icon(Icons.document_scanner),
    );
  }

  Future<void> _scanAndSendDocument(BuildContext context) async {
    final doc = await _scanDocument(context);
    if (doc == null) return;
    await _sendFile(doc);
  }

  Future<File?> _scanDocument(BuildContext context) async {
    try {
      return await DocumentScannerFlutter.launch(context);
    } on PlatformException {
      // 'Failed to get document path or operation cancelled!';
      // TODO: add logging or something
    }
    return null;
  }

  Future<void> _sendFile(File file) async {
    var req = http.MultipartRequest('POST', _urls.upload());
    req.files.add(multipartFile(file));
    final res = await req.send();
    print(res.statusCode);
  }

  http.MultipartFile multipartFile(File file) {
    return http.MultipartFile('document', stream(file), length(file),
        filename: name(file));
  }

  Stream<Uint8List> stream(File file) {
    return file.readAsBytes().asStream();
  }

  int length(File file) {
    return file.lengthSync();
  }

  String name(File file) {
    return file.path.split('/').last;
  }
}
