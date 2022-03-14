import 'package:logging/logging.dart';

mixin Log {
  Logger get log => Logger('$runtimeType');
}
