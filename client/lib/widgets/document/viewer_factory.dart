import 'package:cached_network_image/cached_network_image.dart';
import 'package:dox/screens/incorrect_file.dart';
import 'package:dox/utilities/filetype.dart';
import 'package:dox/widgets/document/image_viewer.dart';
import 'package:dox/widgets/document/pdf_viewer.dart';
import 'package:flutter/material.dart';

class ViewerFactory {
  ViewerFactory._();

  static Widget from(Uri uri) {
    switch (uri.filetype()) {
      case Filetype.image:
        return ImageViewer(
          imageProvider: CachedNetworkImageProvider(uri.toString()),
        );
      case Filetype.pdf:
        return PdfViewer(fileUrl: uri);
      default:
        return const IncorrectFileScreen();
    }
  }
}
