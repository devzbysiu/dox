import 'package:dox/models/document.dart';
import 'package:dox/utilities/filetype.dart';
import 'package:dox/widgets/document/image_viewer.dart';
import 'package:dox/widgets/document/pdf_viewer.dart';
import 'package:flutter/material.dart';

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
        return ImageViewer(
          imageProvider: NetworkImage(doc.fileUrl.toString()),
        );
      case Filetype.pdf:
        return PdfViewer(fileUrl: doc.fileUrl);
      default:
        // TODO: Add some default view
        throw Exception('Filetype not supported');
    }
  }
}
