import 'dart:io';

import 'package:document_scanner_flutter/document_scanner_flutter.dart';
import 'package:dox/utilities/log.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

class DocScanService with Log {
  const DocScanService();

  Future<File?> scanImage(BuildContext context) async {
    try {
      log.fine('launching DocumentScannerFlutter');
      return await DocumentScannerFlutter.launch(context);
    } on PlatformException {
      log.warning('failed to get document path or operation cancelled');
    }
    log.fine('document not scanned, returning null');
    return null;
  }

  Future<File?> pickPdf() async {
    log.fine('picking PDF');
    final result = await FilePicker.platform.pickFiles(
      type: FileType.custom,
      allowedExtensions: ['pdf'],
    );
    if (result == null || result.files.single.path == null) return null;
    final path = result.files.single.path!;
    log.fine('picked file: "$path"');
    return File(path);
  }
}
