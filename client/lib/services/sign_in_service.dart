import 'package:dox/utilities/log.dart';
import 'package:google_sign_in/google_sign_in.dart';

class SignInService with Log {
  SignInService();

  String _idToken = '';

  Future<void> signIn() async {
    final signIn = GoogleSignIn();
    GoogleSignInAccount? account = await signIn.signInSilently();
    account ??= await signIn.signIn();
    final auth = await account!.authentication;
    _idToken = auth.idToken!;
  }

  String get idToken {
    return _idToken;
  }
}
