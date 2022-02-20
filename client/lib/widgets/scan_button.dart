import 'dart:io';

import 'package:document_scanner_flutter/document_scanner_flutter.dart';
import 'package:dox/utilities/dox_service.dart';
import 'package:dox/utilities/theme.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:motion_toast/motion_toast.dart';
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
      closedForegroundColor: onPrimary(context),
      openForegroundColor: secondary(context),
      closedBackgroundColor: onPrimary(context),
      openBackgroundColor: secondary(context),
      speedDialChildren: [
        SpeedDialChild(
          child: Icon(Icons.camera_alt, color: onPrimary(context)),
          foregroundColor: secondary(context),
          backgroundColor: primary(context),
          label: 'Scan document',
          onPressed: () => _scanAndSendImage(context),
        ),
        SpeedDialChild(
          child: const Icon(Icons.picture_as_pdf),
          foregroundColor: secondary(context),
          backgroundColor: primary(context),
          label: 'Pick PDF',
          onPressed: () => _pickAndSendPdf(context),
        ),
      ],
    );
  }

  Future<void> _scanAndSendImage(BuildContext context) async {
    final doc = await _scanImage(context);
    if (doc == null) return;
    await _sendAndRefreshList(doc, context);
  }

  Future<File?> _scanImage(BuildContext context) async {
    try {
      return await DocumentScannerFlutter.launch(context);
    } on PlatformException {
      // 'Failed to get document path or operation cancelled!';
      // TODO: add logging or something
    }
    return null;
  }

  Future<void> _sendAndRefreshList(File doc, BuildContext context) async {
    try {
      await _uploadAndShowToast(doc, context);
      Future.delayed(const Duration(seconds: 2), () {
        onScanned();
      });
    } on Exception {
      _showUploadFailed(context);
    }
  }

  Future<void> _uploadAndShowToast(File doc, BuildContext context) async {
    final resp = await _dox.uploadDoc(doc);
    if (resp.statusCode != 201) {
      _showUploadFailed(context);
      return;
    }
    _showUploadSuccessful(context);
  }

  void _pickAndSendPdf(BuildContext context) async {
    final doc = await _pickPdf();
    if (doc == null) return;
    await _sendAndRefreshList(doc, context);
  }

  Future<File?> _pickPdf() async {
    final result = await FilePicker.platform
        .pickFiles(type: FileType.custom, allowedExtensions: ['pdf']);
    if (result == null || result.files.single.path == null) return null;
    return File(result.files.single.path!);
  }

  void _showUploadFailed(BuildContext context) {
    MotionToast(
      title: const Text('Error'),
      description: const Text('Failed to upload file'),
      icon: Icons.error,
      primaryColor: primary(context),
    ).show(context);
  }

  void _showUploadSuccessful(BuildContext context) {
    MotionToast.success(
            title: const Text('Success'),
            description: const Text('File uploaded successfully'))
        .show(context);
  }
}
