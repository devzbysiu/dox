import 'package:dox/utilities/filetype.dart';

class Document {
  final Uri fileUrl;

  final Uri thumbnailUrl;

  const Document(this.fileUrl, this.thumbnailUrl);

  bool isSupported() {
    final docType = fileUrl.filetype();
    final thumbnailType = thumbnailUrl.filetype();
    return (docType.isImage || docType.isPdf) && thumbnailType.isImage;
  }
}
