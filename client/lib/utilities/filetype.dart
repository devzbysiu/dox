import 'package:path/path.dart' as path;

enum Filetype { image, pdf, other }

extension UriExt on Uri {
  Filetype filetype() {
    switch (path.extension(this.path)) {
      case '.jpg':
      case '.jpeg':
      case '.webp':
      case '.png':
        return Filetype.image;
      case '.pdf':
        return Filetype.pdf;
      default:
        return Filetype.other;
    }
  }
}


extension FiletypeExt on Filetype {
  bool get isImage => this == Filetype.image;
  bool get isPdf => this == Filetype.pdf;
}
