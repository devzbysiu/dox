import 'package:dox/models/docs_state.dart';
import 'package:dox/models/document.dart';
import 'package:dox/utilities/connection.dart';
import 'package:dox/utilities/log.dart';
import 'package:dox/widgets/document/openable_document.dart';
import 'package:flutter/cupertino.dart';
import 'package:provider/provider.dart';

// ignore: must_be_immutable
class OpenableImageList extends StatelessWidget with Log {
  const OpenableImageList({
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final docsState = context.watch<DocsState>();
    final connection = context.watch<Connection>();
    connection.stream.listen((data) {
      log.fine('received data: $data');
      if (data == 'new-doc') {
        log.fine('new doc event received, calling handler');
        docsState.refresh();
      }
    });
    return ListView(
      keyboardDismissBehavior: ScrollViewKeyboardDismissBehavior.onDrag,
      children: _buildOpenableDocuments(docsState.suggestions),
    );
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
