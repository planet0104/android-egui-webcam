
#cargo install --git https://github.com/tauri-apps/cargo-mobile2
# export https_proxy=http://192.168.1.11:6677;export http_proxy=http://192.168.1.11:6677;export all_proxy=socks5://192.168.1.11:6677

# 下载 android studio linux
# wget https://redirector.gvt1.com/edgedl/android/studio/ide-zips/2023.2.1.23/android-studio-2023.2.1.23-linux.tar.gz
# chmod +x ./android-studio-2023.2.1.23-linux.tar.gz 
# tar -zxvf ./android-studio-2023.2.1.23-linux.tar.gz 
# 下载 android sdk
# wget https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip?hl=zh-cn
# mv commandlinetools-linux-11076708_latest.zip\?hl\=zh-cn commandlinetools-linux-11076708_latest.zip
# sudo apt install unzip
# unzip commandlinetools-linux-11076708_latest.zip
# export JAVA_HOME="$HOME/android-studio/jbr/"
# ./cmdline-tools/bin/sdkmanager --list --sdk_root=$HOME/AndroidSdk
# ./cmdline-tools/bin/sdkmanager "platform-tools" "platforms;android-33" --sdk_root=$HOME/AndroidSdk
#  ./cmdline-tools/bin/sdkmanager --install "ndk;21.3.6528147" --sdk_root=$HOME/AndroidSdk
# export GRADLE_OPTS="-Dhttp.proxyHost=192.168.1.11 -Dhttp.proxyPort=6677 -Dhttps.proxyHost=192.168.1.11 -Dhttps.proxyPort=6677"

export JAVA_HOME="$HOME/android-studio/jbr/"
export ANDROID_HOME="$HOME/AndroidSdk"
export NDK_HOME="$ANDROID_HOME/ndk/21.3.6528147"

cargo android apk build