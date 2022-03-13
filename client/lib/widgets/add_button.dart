import 'dart:io';

import 'package:document_scanner_flutter/document_scanner_flutter.dart';
import 'package:dox/models/docs_state.dart';
import 'package:dox/utilities/log.dart';
import 'package:dox/utilities/theme.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:motion_toast/motion_toast.dart';
import 'package:provider/provider.dart';
import 'package:simple_speed_dial/simple_speed_dial.dart';

class AddButton extends StatelessWidget with Log {
  const AddButton({
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return SpeedDial(
      child: const Icon(Icons.add),
      closedForegroundColor: onPrimary(context),
      openForegroundColor: onPrimary(context),
      closedBackgroundColor: primary(context),
      openBackgroundColor: secondary(context),
      speedDialChildren: [
        _buildScanButton(context),
        _buildPdfButton(context),
      ],
    );
  }

  SpeedDialChild _buildScanButton(BuildContext context) {
    return SpeedDialChild(
      child: Icon(Icons.camera_alt, color: onPrimary(context)),
      foregroundColor: secondary(context),
      backgroundColor: primary(context),
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
    if (await _docsModel(context).newDoc(doc)) {
      log.fine('uploaded successfully');
      _showUploadSuccessful(context);
      return;
    }
    log.warning('upload failed');
    _showUploadFailed(context);
  }

  DocsState _docsModel(BuildContext context) {
    return Provider.of<DocsState>(context, listen: false);
  }

  void _showUploadFailed(BuildContext context) {
    log.fine('showing failure toast');
    MotionToast(
      title: const Text('Error'),
      description: const Text('Failed to upload file'),
      icon: Icons.error,
      primaryColor: primary(context),
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
      child: Icon(Icons.picture_as_pdf, color: onPrimary(context)),
      foregroundColor: secondary(context),
      backgroundColor: primary(context),
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
