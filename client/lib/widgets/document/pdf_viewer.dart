import 'package:dox/widgets/document/document_viewer.dart';
import 'package:flutter/material.dart';
import 'package:syncfusion_flutter_pdfviewer/pdfviewer.dart';

class PdfViewer extends DocumentViewer {
  const PdfViewer({
    Key? key,
    required this.fileUrl,
  }) : super(key: key);

  final Uri fileUrl;

  @override
  Widget viewer(BuildContext context) {
    return SfPdfViewer.network(fileUrl.toString());
  }
}
