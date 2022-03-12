import 'package:dox/models/docs_state.dart';
import 'package:dox/models/document.dart';
import 'package:dox/utilities/filetype.dart';
import 'package:dox/widgets/document/openable_document.dart';
import 'package:flutter/cupertino.dart';
import 'package:provider/provider.dart';

// ignore: must_be_immutable
class OpenableImageList extends StatelessWidget {
  const OpenableImageList({
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Consumer<DocsState>(
      builder: (context, model, _) => ListView(
        children: _buildOpenableImages(model),
      ),
    );
  }

  List<Widget> _buildOpenableImages(DocsState model) {
    final docUrls = model.suggestions.where(_isSupportedFiletype).toList();
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
