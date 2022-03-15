import 'package:dox/services/connection_service.dart';
import 'package:dox/services/docs_service.dart';
import 'package:dox/utilities/config.dart';
import 'package:dox/utilities/events_stream.dart';
import 'package:dox/utilities/urls.dart';
import 'package:get_it/get_it.dart';

final getIt = GetIt.instance;

Future<void> setupServices() async {
  getIt.registerSingleton<Config>(await Config.init());
  getIt.registerSingleton<Urls>(Urls());
  getIt.registerSingleton<Events>(Events());
  getIt.registerSingleton<DocsService>(DocsService());
  getIt.registerSingleton<ConnService>(ConnService());
}
