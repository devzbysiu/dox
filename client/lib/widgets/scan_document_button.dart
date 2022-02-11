import 'dart:io';

import 'package:document_scanner_flutter/document_scanner_flutter.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

class ScanDocumentButton extends StatelessWidget {
  const ScanDocumentButton({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return FloatingActionButton(
      onPressed: () async {
        try {
          File? doc = await DocumentScannerFlutter.launch(context);
        } on PlatformException {
          // 'Failed to get document path or operation cancelled!';
        }
      },
      backgroundColor: Colors.orange,
      child: const Icon(Icons.document_scanner),
    );
  }
}
