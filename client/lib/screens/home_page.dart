import 'package:dox/models/search_model.dart';
import 'package:dox/utilities/urls.dart';
import 'package:dox/widgets/openable_image_list.dart';
import 'package:dox/widgets/scan_document_button.dart';
import 'package:dox/widgets/search_input.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

class HomePage extends StatelessWidget {
  late final Urls _urls;

  HomePage(Urls urls, {Key? key}) : super(key: key) {
    _urls = urls;
  }

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
        onTap: () => _hideKeyboard(),
        onVerticalDragDown: (_) => _hideKeyboard(),
        child: Scaffold(
            floatingActionButton: ScanDocumentButton(_urls),
            body: Consumer<SearchModel>(
              builder: (context, model, _) => Column(
                children: <Widget>[
                  Padding(
                    padding: const EdgeInsets.all(8.0),
                    child: SearchInput(onQueryChanged: model.onQueryChanged),
                  ),
                  Expanded(child: OpenableImageList(docUrls: model.docUrls)),
                ],
              ),
            )));
  }

  void _hideKeyboard() {
    FocusManager.instance.primaryFocus?.unfocus();
  }
}
