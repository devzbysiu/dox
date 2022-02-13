import 'package:dox/models/search_model.dart';
import 'package:dox/utilities/dox_service.dart';
import 'package:dox/widgets/openable_image_list.dart';
import 'package:dox/widgets/scan_button.dart';
import 'package:dox/widgets/search_input.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

class HomePage extends StatelessWidget {
  late final DoxService _dox;

  HomePage(DoxService dox, {Key? key}) : super(key: key) {
    _dox = dox;
  }

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
        onTap: () => _hideKeyboard(),
        onVerticalDragDown: (_) => _hideKeyboard(),
        child: Scaffold(
          backgroundColor: Colors.white,
          body: NestedScrollView(
            headerSliverBuilder:
                (BuildContext context, bool innerBoxIsScrolled) {
              return <Widget>[
                SliverAppBar(
                  title: const Text(
                    'Dox',
                    style: TextStyle(color: Colors.white),
                  ),
                  expandedHeight: 220.0,
                  flexibleSpace: FlexibleSpaceBar(
                      centerTitle: true,
                      background: Image.asset(
                        'assets/app-bar.webp',
                        fit: BoxFit.cover,
                      )),
                ),
              ];
            },
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
            ),
          ),
          floatingActionButton: ScanButton(_dox),
        ));
  }

  void _hideKeyboard() {
    FocusManager.instance.primaryFocus?.unfocus();
  }
}
