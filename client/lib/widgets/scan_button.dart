import 'dart:io';

import 'package:document_scanner_flutter/document_scanner_flutter.dart';
import 'package:dox/utilities/dox_service.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

class ScanButton extends StatelessWidget {
  late final DoxService _dox;

  late final Function onScanned;

  ScanButton(DoxService dox, {Key? key, required this.onScanned})
      : super(key: key) {
    _dox = dox;
  }

  @override
  Widget build(BuildContext context) {
    return FloatingActionButton(
      shape: const RoundedRectangleBorder(
          borderRadius: BorderRadius.all(Radius.circular(15))),
      onPressed: () => _scanAndSendDocument(context),
      backgroundColor: Colors.purple,
      child: const Icon(Icons.document_scanner, color: Colors.white),
    );
  }

  Future<void> _scanAndSendDocument(BuildContext context) async {
    final doc = await _scanDocument(context);
    if (doc == null) return;
    try {
      // TODO: the receiving of the image is broken
      await _dox.uploadDoc(doc);
    } on SocketException {
      // nothing
    }
    Future.delayed(const Duration(seconds: 1), () {
      onScanned();
    });
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
