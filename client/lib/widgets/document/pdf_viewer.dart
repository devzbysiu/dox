import 'package:dox/widgets/document/document_viewer.dart';
import 'package:flutter/material.dart';
import 'package:syncfusion_flutter_pdfviewer/pdfviewer.dart';

class PdfViewer extends DocumentViewer {
  const PdfViewer({
    super.key,
    required this.fileUrl,
    required this.headers,
  });

  final Uri fileUrl;

  final Map<String, String> headers;

  @override
  Widget viewer(BuildContext context) {
    return SfPdfViewer.network(fileUrl.toString(), headers: headers);
  }
}
