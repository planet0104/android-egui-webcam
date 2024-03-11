:: 如果android build失败，尝试安装最新的AndroidStudio
:: 设置 JAVA_HOME = "C:\Program Files\Android\Android Studio\jbr"
:: 设置最新的环境变量 ANDROID_HOME = "C:\Users\planet\AppData\Local\Android\Sdk"
:: NDK_HOME = "C:\Users\planet\AppData\Local\Android\Sdk\ndk\26.1.10909125"

:: //在 manifest 中添加 权限: <uses-permission android:name="android.permission.CAMERA" />
cargo android run