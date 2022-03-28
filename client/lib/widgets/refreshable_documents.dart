import 'package:dox/models/docs_state.dart';
import 'package:dox/widgets/openable_image_list.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

class RefreshableDocumentsList extends StatelessWidget {
  const RefreshableDocumentsList({
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Expanded(
      child: RefreshIndicator(
        child: const OpenableImageList(),
        onRefresh: () => _refreshDocs(context),
      ),
    );
  }

  Future<void> _refreshDocs(BuildContext context) async {
    final state = context.read<DocsState>();
    await Future.delayed(const Duration(seconds: 1), state.refresh);
  }
}
