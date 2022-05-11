import 'package:dox/services/lifecycle_service.dart';
import 'package:dox/utilities/theme.dart';
import 'package:dox/widgets/add_button.dart';
import 'package:dox/widgets/app_bar.dart';
import 'package:dox/widgets/refreshable_documents.dart';
import 'package:dox/widgets/search_input.dart';
import 'package:flutter/material.dart';

class HomePage extends StatelessWidget {
  const HomePage({
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Lifecycle(
        child: Scaffold(
      backgroundColor: context.background,
      body: NestedScrollView(
        headerSliverBuilder: (_ctx, _) => [const ScrollableAppBar()],
        body: Column(
          children: const [
            Padding(
              padding: EdgeInsets.all(8.0),
              child: SearchInput(),
            ),
            RefreshableDocumentsList(),
          ],
        ),
      ),
      floatingActionButton: AddButton(),
    ));
  }
}
