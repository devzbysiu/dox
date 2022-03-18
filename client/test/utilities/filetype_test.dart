import 'package:dox/utilities/filetype.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('Uri to supported filetypes returns correct filetype', () {
    final extToFiletype = {
      'jpg': Filetype.image,
      'jpeg': Filetype.image,
      'webp': Filetype.image,
      'png': Filetype.image,
      'pdf': Filetype.pdf,
    };
    extToFiletype.forEach((input, expected) {
      test('Uri to $input file returns correct filetype', () {
        // given
        final uri = Uri.parse("http://some-address.com/file.$input");

        // when
        final filetype = uri.filetype();

        // then
        expect(filetype, expected);
      });
    });
  });


}
