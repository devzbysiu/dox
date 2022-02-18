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
    switch (_filetype()) {
      case _Filetype.image:
        return NetworkImage(url.toString());
      case _Filetype.pdf:
        return const AssetImage('assets/pdf-icon.webp');
      default:
        // TODO: it should be logged and failed in a safe way
        throw Exception('Not supported file extension');
    }
  }

  Image _img() {
    switch (_filetype()) {
      case _Filetype.image:
        return Image.network(
          url.toString(),
          width: 350.0,
          loadingBuilder: (_, child, chunk) =>
              chunk != null ? const Text("loading") : child,
        );
      case _Filetype.pdf:
        return Image.asset('assets/pdf-icon.webp', width: 350.0);
      default:
        // TODO: it should be logged and failed in a safe way
        throw Exception('${_filetype()} not supported');
    }
  }

  _Filetype _filetype() {
    switch (path.extension(url.path)) {
      case ".jpg":
      case ".jpeg":
      case ".webp":
      case ".png":
        return _Filetype.image;
      case ".pdf":
        return _Filetype.pdf;
      default:
        return _Filetype.other;
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

enum _Filetype { image, pdf, other }
