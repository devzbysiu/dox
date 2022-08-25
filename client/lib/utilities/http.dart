import 'package:dox/services/sign_in_service.dart';
import 'package:dox/utilities/log.dart';
import 'package:dox/utilities/service_locator.dart';
import 'package:http/http.dart' as http;

class AuthenticatedClient with Log {
  static Future<AuthenticatedClient> init({SignInService? signIn}) async {
    final signInService = signIn ?? getIt<SignInService>();
    await signInService.signIn();
    _singleton ??= AuthenticatedClient._(signInService);
    return _singleton!;
  }

  AuthenticatedClient._(SignInService signInService) {
    _signInService = signInService;
  }

  late final SignInService _signInService;

  static AuthenticatedClient? _singleton;

  Future<http.Response> get(Uri url) async {
    return http.get(url, headers: {'authorization': _signInService.idToken});
  }

  Future<http.Response> post(Uri url,
      {Map<String, String>? headers, Object? body}) {
    final authenticatedHeaders = headers ?? {};
    authenticatedHeaders.addAll({'authorization': _signInService.idToken});
    return http.post(url, headers: authenticatedHeaders, body: body);
  }
}
