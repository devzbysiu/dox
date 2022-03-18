import 'package:dox/utilities/filetype.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('Uri to file returns correct filetype', () {
    // given
    final uri = Uri.parse("http://some-address.com/file.jpg");

    // when
    final filetype = uri.filetype();

    // then
    expect(filetype, Filetype.image);
  });
}
