import 'package:dox/models/document.dart';
import 'package:dox/widgets/document/viewer_factory.dart';
import 'package:dox/widgets/document/hero_image.dart';
import 'package:flutter/material.dart';

class OpenableDocument extends StatelessWidget {
  final Document doc;

  const OpenableDocument({Key? key, required this.doc}) : super(key: key);

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
