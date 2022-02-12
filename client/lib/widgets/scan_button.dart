import 'dart:io';

import 'package:document_scanner_flutter/document_scanner_flutter.dart';
import 'package:dox/utilities/dox_service.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

class ScanButton extends StatelessWidget {
  late final DoxService _dox;

  ScanButton(DoxService dox, {Key? key}) : super(key: key) {
    _dox = dox;
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
    await _dox.uploadDoc(doc);
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
}
