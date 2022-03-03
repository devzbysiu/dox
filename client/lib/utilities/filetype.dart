import 'package:path/path.dart' as path;

enum Filetype { image, pdf, other }

Filetype filetype(Uri url) {
  switch (path.extension(url.path)) {
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
