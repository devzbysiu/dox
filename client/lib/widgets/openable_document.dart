import 'package:dox/models/document.dart';
import 'package:dox/utilities/filetype.dart';
import 'package:dox/utilities/theme.dart';
import 'package:flutter/material.dart';
import 'package:photo_view/photo_view.dart';
import 'package:syncfusion_flutter_pdfviewer/pdfviewer.dart';

class OpenableDocument extends StatelessWidget {
  final Document doc;

  const OpenableDocument({Key? key, required this.doc}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Center(
      child: GestureDetector(
        onTap: () {
          Navigator.push(
            context,
            MaterialPageRoute(
              builder: (context) => _documentViewer(),
            ),
          );
        },
        child: Hero(
          tag: doc.thumbnailUrl.toString(),
          child: Container(
            decoration: const BoxDecoration(
              borderRadius: BorderRadius.all(Radius.circular(15)),
              color: Color.fromRGBO(242, 242, 246, 1),
            ),
            padding: const EdgeInsets.all(20),
            child: Image.network(
              doc.thumbnailUrl.toString(),
              width: 350.0,
              loadingBuilder: (_, child, chunk) =>
                  chunk != null ? const Text("loading") : child,
            ),
          ),
        ),
      ),
    );
  }

  Widget _documentViewer() {
    switch (filetype(doc.fileUrl)) {
      case Filetype.image:
        return _ImageViewer(
          imageProvider: NetworkImage(doc.fileUrl.toString()),
        );
      case Filetype.pdf:
        return _PdfViewer(fileUrl: doc.fileUrl);
      default:
        // TODO: Add some default view
        throw Exception('Filetype not supported');
    }
  }
}

abstract class _DocumentViewer extends StatelessWidget {
  const _DocumentViewer({
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Container(
      constraints: BoxConstraints.expand(
        height: MediaQuery.of(context).size.height,
      ),
      child: viewer(context),
    );
  }

  Widget viewer(BuildContext context);
}

class _ImageViewer extends _DocumentViewer {
  final ImageProvider imageProvider;

  const _ImageViewer({required this.imageProvider});

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

class _PdfViewer extends _DocumentViewer {
  final Uri fileUrl;

  const _PdfViewer({required this.fileUrl});

  @override
  Widget viewer(BuildContext context) {
    return SfPdfViewer.network(fileUrl.toString());
  }
}
