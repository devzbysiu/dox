import 'package:dox/services/connection_service.dart';
import 'package:dox/services/doc_scan_service.dart';
import 'package:dox/services/docs_service.dart';
import 'package:dox/utilities/config.dart';
import 'package:dox/utilities/events_stream.dart';
import 'package:dox/utilities/urls.dart';
import 'package:get_it/get_it.dart';

final getIt = GetIt.instance;

Future<void> setupServices({
  Config? configOverride,
  Events? eventsOverride,
}) async {
  getIt.registerSingleton<Config>(configOverride ?? await ConfigImpl.init());
  getIt.registerSingleton<Urls>(Urls());
  getIt.registerSingleton<Events>(eventsOverride ?? EventsImpl());
  getIt.registerSingleton<DocsService>(DocsService());
  getIt.registerSingleton<ConnService>(ConnService());
  getIt.registerSingleton<DocScanService>(const DocScanService());
}
