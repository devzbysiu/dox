import 'package:dox/models/document.dart';
import 'package:dox/utilities/filetype.dart';
import 'package:dox/widgets/document/image_hero.dart';
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
        child: ImageHero(doc: doc),
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
