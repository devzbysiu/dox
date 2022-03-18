import 'package:dox/models/docs_state.dart';
import 'package:dox/models/document.dart';
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
    final suggestions = context.select((DocsStateImpl docs) => docs.suggestions);
    return ListView(children: _buildOpenableDocuments(suggestions));
  }

  List<Widget> _buildOpenableDocuments(List<Document> suggestions) {
    final docUrls = suggestions.where((d) => d.isSupported()).toList();
    return docUrls.map(_buildOpenableDocument).toList();
  }

  Widget _buildOpenableDocument(Document doc) {
    return Padding(
      padding: const EdgeInsets.all(15),
      child: OpenableDocument(doc: doc),
    );
  }
}
