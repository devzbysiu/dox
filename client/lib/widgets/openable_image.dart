import 'package:flutter/material.dart';
import 'package:photo_view/photo_view.dart';
import 'package:path/path.dart' as path;

class OpenableImage extends StatelessWidget {
  final Uri url;

  const OpenableImage({Key? key, required this.url}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Center(
      child: GestureDetector(
        onTap: () {
          Navigator.push(
            context,
            MaterialPageRoute(
              builder: (context) => HeroPhotoViewRouteWrapper(
                imageProvider: _imgProvider(),
                backgroundDecoration: const BoxDecoration(color: Colors.white),
              ),
            ),
          );
        },
        child: Hero(
          tag: url.toString(),
          child: Container(
            decoration: const BoxDecoration(
                borderRadius: BorderRadius.all(Radius.circular(15)),
                color: Color.fromRGBO(242, 242, 246, 1)),
            padding: const EdgeInsets.all(20),
            child: _img(),
          ),
        ),
      ),
    );
  }

  ImageProvider _imgProvider() {
    final extension = path.extension(url.path);
    switch (extension) {
      case ".jpg":
      case ".jpeg":
      case ".webp":
      case ".png":
        return NetworkImage(url.toString());
      case ".pdf":
        return const AssetImage('assets/pdf-icon.webp');
      default:
        // TODO: it should be logged and failed in a safe way
        throw Exception('Not supported file extension');
    }
  }

  Image _img() {
    final extension = path.extension(url.path);
    switch (extension) {
      case ".jpg":
      case ".jpeg":
      case ".webp":
      case ".png":
        return Image.network(
          url.toString(),
          width: 350.0,
          loadingBuilder: (_, child, chunk) =>
              chunk != null ? const Text("loading") : child,
        );
      case ".pdf":
        return Image.asset('assets/pdf-icon.webp', width: 350.0);
      default:
        // TODO: it should be logged and failed in a safe way
        throw Exception('Not supported file extension');
    }
  }
}

class HeroPhotoViewRouteWrapper extends StatelessWidget {
  final ImageProvider imageProvider;
  final BoxDecoration? backgroundDecoration;

  const HeroPhotoViewRouteWrapper({
    Key? key,
    required this.imageProvider,
    this.backgroundDecoration,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Container(
      constraints: BoxConstraints.expand(
        height: MediaQuery.of(context).size.height,
      ),
      child: PhotoView(
        imageProvider: imageProvider,
        backgroundDecoration: backgroundDecoration,
      ),
    );
  }
}
