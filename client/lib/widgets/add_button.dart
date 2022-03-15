import 'dart:io';

import 'package:document_scanner_flutter/document_scanner_flutter.dart';
import 'package:dox/services/docs_service.dart';
import 'package:dox/utilities/log.dart';
import 'package:dox/utilities/service_locator.dart';
import 'package:dox/utilities/theme.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:motion_toast/motion_toast.dart';
import 'package:simple_speed_dial/simple_speed_dial.dart';

class AddButton extends StatelessWidget with Log {
  late final DocsService _docsService;

  AddButton({
    Key? key,
    DocsService? docsService,
  }) : super(key: key) {
    _docsService = docsService ?? getIt<DocsService>();
  }

  @override
  Widget build(BuildContext context) {
    return SpeedDial(
      child: const Icon(Icons.add),
      closedForegroundColor: context.onPrimary,
      openForegroundColor: context.onPrimary,
      closedBackgroundColor: context.primary,
      openBackgroundColor: context.secondary,
      speedDialChildren: [
        _buildScanButton(context),
        _buildPdfButton(context),
      ],
    );
  }

  SpeedDialChild _buildScanButton(BuildContext context) {
    return SpeedDialChild(
      child: Icon(Icons.camera_alt, color: context.onPrimary),
      foregroundColor: context.secondary,
      backgroundColor: context.primary,
      label: 'Scan document',
      onPressed: () => _scanAndSendImage(context),
    );
  }

  Future<void> _scanAndSendImage(BuildContext context) async {
    log.fine('scanning and sending an image');
    final doc = await _scanImage(context);
    if (doc == null) return;
    await _send(doc, context);
  }

  Future<File?> _scanImage(BuildContext context) async {
    try {
      log.fine('launching DocumentScannerFlutter');
      return await DocumentScannerFlutter.launch(context);
    } on PlatformException {
      log.warning('failed to get document path or operation cancelled');
    }
    log.fine('document not scanned, returning null');
    return null;
  }

  Future<void> _send(File doc, BuildContext context) async {
    try {
      log.fine('sending file');
      await _uploadAndShowToast(doc, context);
    } on Exception {
      _showUploadFailed(context);
    }
  }

  Future<void> _uploadAndShowToast(File doc, BuildContext context) async {
    log.fine('uploading file: "${doc.path}"');
    final res = await _docsService.uploadDoc(doc);
    if (res.statusCode == 201) {
      log.fine('uploaded successfully');
      _showUploadSuccessful(context);
      return;
    }
    log.warning('upload failed');
    _showUploadFailed(context);
  }

  void _showUploadFailed(BuildContext context) {
    log.fine('showing failure toast');
    MotionToast(
      title: const Text('Error'),
      description: const Text('Failed to upload file'),
      icon: Icons.error,
      primaryColor: context.primary,
    ).show(context);
  }

  void _showUploadSuccessful(BuildContext context) {
    log.fine('showing success toast');
    MotionToast.success(
      title: const Text('Success'),
      description: const Text('File uploaded successfully'),
    ).show(context);
  }

  SpeedDialChild _buildPdfButton(BuildContext context) {
    return SpeedDialChild(
      child: Icon(Icons.picture_as_pdf, color: context.onPrimary),
      foregroundColor: context.secondary,
      backgroundColor: context.primary,
      label: 'Pick PDF',
      onPressed: () => _pickAndSendPdf(context),
    );
  }

  void _pickAndSendPdf(BuildContext context) async {
    log.fine('picking and sending PDF');
    final doc = await _pickPdf();
    if (doc == null) return;
    await _send(doc, context);
  }

  Future<File?> _pickPdf() async {
    log.fine('picking PDF');
    final result = await FilePicker.platform
        .pickFiles(type: FileType.custom, allowedExtensions: ['pdf']);
    if (result == null || result.files.single.path == null) return null;
    final path = result.files.single.path!;
    log.fine('picked file: "$path"');
    return File(path);
  }
}
