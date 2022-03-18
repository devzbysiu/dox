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

  group('Uri to unsupported filetypes returns Filetype.other', () {
    final extToFiletype = {
      'abc': Filetype.other,
      '': Filetype.other,
      'docx': Filetype.other,
      'doc': Filetype.other,
      'tex': Filetype.other,
    };
    extToFiletype.forEach((input, expected) {
      test('Uri to $input file returns Filetype.other', () {
        // given
        final uri = Uri.parse("http://some-address.com/file.$input");

        // when
        final filetype = uri.filetype();

        // then
        expect(filetype, expected);
      });
    });
  });

  group('isImage return correct results', () {
    final filetypeToResult = {
      Filetype.image: true,
      Filetype.pdf: false,
      Filetype.other: false,
    };
    filetypeToResult.forEach((input, expected) {
      test('isImage for $input returns $expected', () {
        // given
        final filetype = input;

        // when
        final isImage = filetype.isImage;

        // then
        expect(isImage, expected);
      });
    });
  });

  group('isPdf return correct results', () {
    final filetypeToResult = {
      Filetype.image: false,
      Filetype.pdf: true,
      Filetype.other: false,
    };
    filetypeToResult.forEach((input, expected) {
      test('isPdf for $input returns $expected', () {
        // given
        final filetype = input;

        // when
        final isPdf = filetype.isPdf;

        // then
        expect(isPdf, expected);
      });
    });
  });
}
