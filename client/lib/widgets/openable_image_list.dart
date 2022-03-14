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
    final suggestions = context.select((DocsState docs) => docs.suggestions);
    return ListView(children: _buildOpenableDocuments(suggestions));
  }

  List<Widget> _buildOpenableDocuments(List<Document> suggestions) {
    final docUrls = suggestions.where(_isSupportedFiletype).toList();
    return docUrls.map(_buildOpenableDocument).toList();
  }

  bool _isSupportedFiletype(Document doc) {
    final docType = filetype(doc.fileUrl);
    final thumbnailType = filetype(doc.thumbnailUrl);
    return (docType.isImage || docType.isPdf) && thumbnailType.isImage;
  }

  Widget _buildOpenableDocument(Document doc) {
    return Padding(
      padding: const EdgeInsets.all(15),
      child: OpenableDocument(doc: doc),
    );
  }
}
