import 'package:dox/services/sign_in_service.dart';
import 'package:dox/utilities/log.dart';
import 'package:dox/utilities/service_locator.dart';
import 'package:http/http.dart' as http;

class AuthClient with Log {
  static Future<AuthClient> init({SignInService? signIn}) async {
    final signInService = signIn ?? getIt<SignInService>();
    await signInService.signIn();
    _singleton ??= AuthClient._(signInService);
    return _singleton!;
  }

  AuthClient._(SignInService signInService) {
    _signInService = signInService;
  }

  late final SignInService _signInService;

  static AuthClient? _singleton;

  Future<http.Response> get(Uri url) async {
    return http.get(url, headers: _signInService.authHeaders);
  }

  Future<http.Response> post(Uri url,
      {Map<String, String>? headers, Object? body}) {
    final authenticatedHeaders = headers ?? {};
    authenticatedHeaders.addAll(_signInService.authHeaders);
    return http.post(url, headers: authenticatedHeaders, body: body);
  }
}
