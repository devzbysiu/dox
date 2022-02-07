import 'package:flutter/material.dart';
import 'package:photo_view/photo_view.dart';

class OpenableImage extends StatelessWidget {
  final String url;

  const OpenableImage({Key? key, required this.url}): super(key: key);

  @override
  Widget build(BuildContext context) {
    return Center(
      child: GestureDetector(
        onTap: () {
          Navigator.push(
            context,
            MaterialPageRoute(
              builder: (context) => HeroPhotoViewRouteWrapper(
                imageProvider: NetworkImage(url),
              ),
            ),
          );
        },
        child: Hero(
          tag: "someTag",
          child: Image.network(
            url,
            width: 350.0,
            loadingBuilder: (_, child, chunk) =>
            chunk != null ? const Text("loading") : child,
          ),
        ),
      ),
    );
  }
}

class HeroPhotoViewRouteWrapper extends StatelessWidget {
  final ImageProvider imageProvider;
  final BoxDecoration? backgroundDecoration;

  const HeroPhotoViewRouteWrapper({
    Key? key,
    required this.imageProvider,
    this.backgroundDecoration,
  }): super(key: key);

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
