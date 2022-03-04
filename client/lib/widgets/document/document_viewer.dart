import 'package:flutter/material.dart';

abstract class DocumentViewer extends StatelessWidget {
  const DocumentViewer({
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Container(
      constraints: BoxConstraints.expand(
        height: MediaQuery.of(context).size.height,
      ),
      child: viewer(context),
    );
  }

  Widget viewer(BuildContext context);
}
