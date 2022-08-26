import 'package:cached_network_image/cached_network_image.dart';
import 'package:dox/utilities/theme.dart';
import 'package:dox/widgets/document/document_viewer.dart';
import 'package:flutter/material.dart';
import 'package:photo_view/photo_view.dart';

class ImageViewer extends DocumentViewer {
  const ImageViewer({
    super.key,
    required this.fileUrl,
    required this.headers,
  });

  final Uri fileUrl;

  final Map<String, String> headers;

  @override
  Widget viewer(BuildContext context) {
    return PhotoView(
      imageProvider:
          CachedNetworkImageProvider(fileUrl.toString(), headers: headers),
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
