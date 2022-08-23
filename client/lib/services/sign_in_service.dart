import 'package:dox/utilities/log.dart';
import 'package:google_sign_in/google_sign_in.dart';

class SignInService with Log {
  SignInService();

  late final String _idToken;

  Future<void> signIn() async {
    final signIn = GoogleSignIn();
    final account = await signIn.signInSilently();
    final auth = await account!.authentication;
    _idToken = auth.idToken!;
  }

  String get idToken {
    return _idToken;
  }
}
