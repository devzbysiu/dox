import 'dart:io';

import 'package:document_scanner_flutter/document_scanner_flutter.dart';
import 'package:dox/utilities/dox_service.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:simple_speed_dial/simple_speed_dial.dart';

class ScanButton extends StatelessWidget {
  late final DoxService _dox;

  late final Function onScanned;

  ScanButton(DoxService dox, {Key? key, required this.onScanned})
      : super(key: key) {
    _dox = dox;
  }

  @override
  Widget build(BuildContext context) {
    return SpeedDial(
      child: const Icon(Icons.add),
      closedForegroundColor: Colors.white,
      openForegroundColor: Colors.white,
      closedBackgroundColor: Colors.purple,
      openBackgroundColor: Colors.deepPurple,
      speedDialChildren: [
        SpeedDialChild(
          child: const Icon(Icons.camera_alt, color: Colors.white),
          foregroundColor: Colors.white,
          backgroundColor: Colors.purple,
          label: 'Scan document',
          onPressed: () => _scanAndSendDocument(context),
        ),
        SpeedDialChild(
          child: const Icon(Icons.picture_as_pdf),
          foregroundColor: Colors.white,
          backgroundColor: Colors.purple,
          label: 'Pick PDF',
          onPressed: () => _pickAndSendPdf(),
        ),
      ],
    );
  }

  Future<void> _scanAndSendDocument(BuildContext context) async {
    final doc = await _scanDocument(context);
    if (doc == null) return;
    // TODO: check if upload successful
    await _dox.uploadDoc(doc);
    Future.delayed(const Duration(seconds: 2), () {
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

  void _pickAndSendPdf() async {
    FilePickerResult? result = await FilePicker.platform
        .pickFiles(type: FileType.custom, allowedExtensions: ['pdf']);

    if (result != null) {
      File file = File(result.files.single.path!);
      await _dox.uploadDoc(file);
    } else {
      // User canceled the picker
    }
  }
}
