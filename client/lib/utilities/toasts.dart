import 'package:dox/utilities/theme.dart';
import 'package:flutter/material.dart';
import 'package:motion_toast/motion_toast.dart';

extension BuildContextExt on BuildContext {
  void showSuccessToast(String description) {
    MotionToast.success(
      title: const Text('Success'),
      description: Text(description),
    ).show(this);
  }


  void showFailureToast(String description) {
    MotionToast(
      title: const Text('Error'),
      description: Text(description),
      icon: Icons.error,
      primaryColor: primary,
    ).show(this);
  }
}
