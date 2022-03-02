import 'package:dox/widgets/openable_image.dart';
import 'package:flutter/cupertino.dart';
import 'package:dox/utilities/filetype.dart';

// ignore: must_be_immutable
class OpenableImageList extends StatelessWidget {
  List<Uri> urls = List.empty();

  OpenableImageList({Key? key, required urls}) : super(key: key) {
    this.urls = urls.where(_isSupportedFiletype).toList();
  }

  @override
  Widget build(BuildContext context) {
    return ListView(children: _buildOpenableImages());
  }

  List<Widget> _buildOpenableImages() {
    return urls.map(_buildImage).toList();
  }

  Widget _buildImage(Uri url) {
    return Padding(
      padding: const EdgeInsets.all(15),
      child: OpenableImage(thumbnailUrl: url),
    );
  }
}

bool _isSupportedFiletype(Uri url) {
  final type = filetype(url);
  return type == Filetype.image || type == Filetype.pdf;
}
