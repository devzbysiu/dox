import 'package:dox/services/sign_in_service.dart';
import 'package:dox/utilities/log.dart';
import 'package:dox/utilities/service_locator.dart';
import 'package:http/http.dart' as http;

class AuthenticatedClient with Log {
  AuthenticatedClient({SignInService? signInService}) {
    _signInService = signInService ?? getIt<SignInService>();
    _signInService.signIn();
  }

  late final SignInService _signInService;

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
