import 'package:dox/widgets/openable_image.dart';
import 'package:flutter/cupertino.dart';

class OpenableImageList extends StatelessWidget {
  final List<Uri> docUrls;

  const OpenableImageList({Key? key, required this.docUrls}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return ListView(children: _buildOpenableImages());
  }

  List<Widget> _buildOpenableImages() {
    return docUrls.map(_buildImage).toList();
  }

  Widget _buildImage(Uri url) {
    return Padding(
        padding: const EdgeInsets.all(15), child: OpenableImage(url: url));
  }
}
