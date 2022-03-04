import 'package:dox/utilities/filetype.dart';
import 'package:dox/widgets/document/image_viewer.dart';
import 'package:dox/widgets/document/pdf_viewer.dart';
import 'package:flutter/material.dart';

class ViewerFactory {
  ViewerFactory._();

  static Widget from(Uri uri) {
    switch (filetype(uri)) {
      case Filetype.image:
        return ImageViewer(
          imageProvider: NetworkImage(uri.toString()),
        );
      case Filetype.pdf:
        return PdfViewer(fileUrl: uri);
      default:
        // TODO: Add some default view
        throw Exception('Filetype not supported');
    }
  }
}
