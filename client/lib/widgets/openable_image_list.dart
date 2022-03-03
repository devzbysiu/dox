import 'package:dox/models/document.dart';
import 'package:dox/widgets/document/openable_document.dart';
import 'package:flutter/cupertino.dart';
import 'package:dox/utilities/filetype.dart';

// ignore: must_be_immutable
class OpenableImageList extends StatelessWidget {
  List<Document> docUrls = List.empty();

  OpenableImageList({Key? key, required docUrls}) : super(key: key) {
    this.docUrls = docUrls.where(_isSupportedFiletype).toList();
  }

  @override
  Widget build(BuildContext context) {
    return ListView(children: _buildOpenableImages());
  }

  List<Widget> _buildOpenableImages() {
    return docUrls.map(_buildImage).toList();
  }

  Widget _buildImage(Document doc) {
    return Padding(
      padding: const EdgeInsets.all(15),
      child: OpenableDocument(doc: doc),
    );
  }
}

bool _isSupportedFiletype(Document doc) {
  final docType = filetype(doc.fileUrl);
  return (docType == Filetype.image || docType == Filetype.pdf) &&
      filetype(doc.thumbnailUrl) == Filetype.image;
}
