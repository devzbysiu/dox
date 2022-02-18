import 'package:dox/utilities/filetype.dart';
import 'package:flutter/material.dart';
import 'package:photo_view/photo_view.dart';

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
              builder: (context) => _HeroPhotoViewRouteWrapper(
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
    switch (filetype(url)) {
      case Filetype.image:
        return NetworkImage(url.toString());
      case Filetype.pdf:
        return const AssetImage('assets/pdf-icon.webp');
      default:
        // NOTE: this shouldn't happen as files are filtered earlier
        throw Exception('filetype "${filetype(url)}" is not supported');
    }
  }

  Image _img() {
    switch (filetype(url)) {
      case Filetype.image:
        return Image.network(
          url.toString(),
          width: 350.0,
          loadingBuilder: (_, child, chunk) =>
              chunk != null ? const Text("loading") : child,
        );
      case Filetype.pdf:
        return Image.asset('assets/pdf-icon.webp', width: 350.0);
      default:
        // NOTE: this shouldn't happen as files are filtered earlier
        throw Exception('filetype "${filetype(url)}" is not supported');
    }
  }
}

class _HeroPhotoViewRouteWrapper extends StatelessWidget {
  final ImageProvider imageProvider;
  final BoxDecoration? backgroundDecoration;

  const _HeroPhotoViewRouteWrapper({
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
