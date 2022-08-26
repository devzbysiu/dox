import 'package:dox/utilities/log.dart';
import 'package:google_sign_in/google_sign_in.dart';

class SignInService with Log {
  SignInService();

  Map<String, String> _authHeaders = {};

  Future<void> signIn() async {
    final signIn = GoogleSignIn();
    GoogleSignInAccount? account = await signIn.signInSilently();
    account ??= await signIn.signIn();
    final auth = await account!.authentication;
    _authHeaders = {'authorization': auth.idToken!};
  }

  Map<String, String> get authHeaders => _authHeaders;
}
