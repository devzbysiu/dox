import 'package:dox/utilities/theme.dart';
import 'package:dox/widgets/document/document_viewer.dart';
import 'package:flutter/material.dart';
import 'package:photo_view/photo_view.dart';

class ImageViewer extends DocumentViewer {
  final ImageProvider imageProvider;

  const ImageViewer({Key? key, required this.imageProvider}) : super(key: key);

  @override
  Widget viewer(BuildContext context) {
    return PhotoView(
        imageProvider: imageProvider,
        backgroundDecoration: BoxDecoration(color: onPrimary(context)),
        // TODO: show something better than Placeholder
        loadingBuilder: (context, chunk) =>
            chunk != null ? const Text("loading") : const Placeholder());
  }
}
