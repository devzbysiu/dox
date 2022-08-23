import 'package:dox/services/doc_scan_service.dart';
import 'package:dox/services/docs_service.dart';
import 'package:dox/services/sign_in_service.dart';
import 'package:dox/utilities/config.dart';
import 'package:dox/utilities/http.dart';
import 'package:dox/utilities/urls.dart';
import 'package:get_it/get_it.dart';

final getIt = GetIt.instance;

Future<void> setupServices({
  Config? configOverride,
}) async {
  getIt.registerSingleton<Config>(configOverride ?? await ConfigImpl.init());
  getIt.registerSingleton<Urls>(Urls());
  getIt.registerSingleton<SignInService>(SignInService());
  getIt.registerSingleton<AuthenticatedClient>(AuthenticatedClient());
  getIt.registerSingleton<DocsService>(DocsService());
  getIt.registerSingleton<DocScanService>(const DocScanService());
}
