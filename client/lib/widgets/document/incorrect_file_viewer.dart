import 'package:dox/screens/incorrect_file.dart';
import 'package:dox/widgets/document/document_viewer.dart';
import 'package:flutter/material.dart';

class IncorrectFileViewer extends DocumentViewer {
  const IncorrectFileViewer({
    Key? key,
  }) : super(key: key);

  @override
  Widget viewer(BuildContext context) {
    return const IncorrectFile();
  }
}
