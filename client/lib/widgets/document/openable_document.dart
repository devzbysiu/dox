import 'package:dox/models/document.dart';
import 'package:dox/widgets/document/hero_image.dart';
import 'package:dox/widgets/document/viewer_factory.dart';
import 'package:flutter/material.dart';

class OpenableDocument extends StatelessWidget {
  const OpenableDocument({
    Key? key,
    required this.doc,
  }) : super(key: key);

  final Document doc;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: GestureDetector(
        onTap: () {
          Navigator.push(
            context,
            MaterialPageRoute(
              builder: (_) => ViewerFactory.from(doc.fileUrl),
            ),
          );
        },
        child: HeroImage(doc: doc),
      ),
    );
  }
}
