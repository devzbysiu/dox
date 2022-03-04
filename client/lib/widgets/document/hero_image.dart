import 'package:dox/models/document.dart';
import 'package:flutter/material.dart';

class HeroImage extends StatelessWidget {
  final Document doc;

  const HeroImage({Key? key, required this.doc}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Hero(
      tag: doc.thumbnailUrl.toString(),
      child: Container(
        decoration: const BoxDecoration(
          borderRadius: BorderRadius.all(Radius.circular(15)),
          color: Color.fromRGBO(242, 242, 246, 1),
        ),
        padding: const EdgeInsets.all(20),
        child: Image.network(
          doc.thumbnailUrl.toString(),
          width: 350.0,
          loadingBuilder: (_, child, chunk) =>
              chunk != null ? const Text("loading") : child,
        ),
      ),
    );
  }
}
