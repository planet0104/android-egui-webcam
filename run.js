
const process = require("child_process");

const password = '123456';

console.log('正在签名..');

const cmd = process.exec('apksigner sign --ks test.jks ./gen/android/app/build/outputs/apk/arm64/release/app-arm64-release-unsigned.apk', (error, stdout, stderr) => {
	console.log('apksigner 签名完成 error=', error);
	
	console.log('安装apk...');
	//安装apk
	process.exec('adb install ./gen/android/app/build/outputs/apk/arm64/release/app-arm64-release-unsigned.apk', (error, stdout, stderr) => {
		console.log('adb install 安装完成 error=', error);
		process.exec('adb shell am start -n com.example.android_egui_webcam/android.app.NativeActivity', (error, stdout, stderr) => {
			console.log('apk 启动完成 error=', error);
		});
	});
});

// 输入密码到 cmd
cmd.stdin.write(`${password}\n`);
cmd.stdin.end();