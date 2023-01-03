import 'package:dox/utilities/log.dart';
import 'package:google_sign_in/google_sign_in.dart';

abstract class SignInService {
  Future<void> signIn();
  Map<String, String> get authHeaders;
}

class SignInServiceImpl with Log implements SignInService {
  SignInServiceImpl();

  Map<String, String> _authHeaders = {};

  @override
  Future<void> signIn() async {
    final signIn = GoogleSignIn();
    GoogleSignInAccount? account = await signIn.signInSilently();
    account ??= await signIn.signIn();
    final auth = await account!.authentication;
    _authHeaders = {'authorization': auth.idToken!};
  }

  @override
  Map<String, String> get authHeaders => _authHeaders;
}
