import 'package:dox/utilities/theme.dart';
import 'package:dox/widgets/document/document_viewer.dart';
import 'package:flutter/material.dart';
import 'package:photo_view/photo_view.dart';

class ImageViewer extends DocumentViewer {
  final ImageProvider imageProvider;

  const ImageViewer({
    Key? key,
    required this.imageProvider,
  }) : super(key: key);

  @override
  Widget viewer(BuildContext context) {
    return PhotoView(
      imageProvider: imageProvider,
      backgroundDecoration: BoxDecoration(color: context.onPrimary),
      loadingBuilder: (context, event) => Center(
        child: SizedBox(
          width: 20.0,
          height: 20.0,
          child: CircularProgressIndicator(
            value: event == null
                ? 0
                : event.cumulativeBytesLoaded / event.expectedTotalBytes!,
          ),
        ),
      ),
    );
  }
}
