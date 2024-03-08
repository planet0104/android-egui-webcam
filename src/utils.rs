
#[allow(dead_code)]
#[cfg(target_os = "android")]
pub mod permission{
    use anyhow::Result;
    use jni::{objects::{JObject, JValueGen}, sys::{JNIInvokeInterface_, _jobject, jint}, JavaVM};
    use log::info;
    use winit::platform::android::activity::AndroidApp;

    pub fn sdk_version(app: &AndroidApp) -> Result<i32>{
        unsafe{
            let vm = JavaVM::from_raw(app.vm_as_ptr() as *mut *const JNIInvokeInterface_)?;
            let mut env = vm.attach_current_thread()?;
            Ok(env.get_static_field("android/os/Build$VERSION", "SDK_INT", "I")?.i()?)
        }
    }

    pub fn check_self_permission(app: &AndroidApp, permission:&str) -> Result<bool>{
        unsafe{
            let vm = JavaVM::from_raw(app.vm_as_ptr() as *mut *const JNIInvokeInterface_)?;
            let mut env = vm.attach_current_thread()?;
            info!("check_self_permission 001");
            let granted_int = env.get_static_field("android/content/pm/PackageManager", "PERMISSION_GRANTED", "I")?.i()?;
            info!("check_self_permission 002");
            // 创建Java字符串
            let permission_str = env.new_string(permission)?;
            let activity: JObject<'_> = JObject::from_raw(app.activity_as_ptr() as *mut _jobject);
            info!("check_self_permission 003");
            let result = env.call_method(activity, "checkSelfPermission", "(Ljava/lang/String;)I", &[JValueGen::Object(&JObject::from(permission_str))])?.i()?;
            info!("check_self_permission result={result}");
            Ok(result == granted_int)
        }
    }

    pub fn request_permissions(app: &AndroidApp, permissions:&[&str], request_code: i32) -> Result<()>{
        unsafe{
            let vm = JavaVM::from_raw(app.vm_as_ptr() as *mut *const JNIInvokeInterface_)?;

            info!("request_permissions 001");

            let mut env = vm.attach_current_thread()?;
            
            info!("request_permissions 002");

            let activity: JObject<'_> = JObject::from_raw(app.activity_as_ptr() as *mut _jobject);
            info!("request_permissions 003");

            // 创建一个Java String数组
            let permission_count = permissions.len() as jint;
            info!("request_permissions permission_count = {permission_count}");
            info!("request_permissions 004");
            let java_permission_array = env.new_object_array(permission_count, "java/lang/String", JObject::null())?;
            info!("request_permissions 005");
            for (index, permission) in permissions.iter().enumerate() {
                info!("request_permissions new string: {permission}");
                let permission_str = env.new_string(*permission)?;
                env.set_object_array_element(&java_permission_array, index as jint, permission_str)?;
            }
            info!("request_permissions 006");

            // 调用requestPermissions方法
            let _ = env.call_method(
                activity,
                "requestPermissions",
                "([Ljava/lang/String;I)V",
                &[JValueGen::Object(&JObject::from(java_permission_array)), request_code.into()],
            )?;
        }
        Ok(())
    }

    pub fn request_camera_permission(app: &AndroidApp) -> Result<()>{
        let sdk_version = sdk_version(app)?;
        info!("sdk version:{sdk_version}");
        let permission = "android.permission.CAMERA";
        if sdk_version > 23{
            if !check_self_permission(app, permission)?{
                request_permissions(app, &[permission], 100)?;
            }
        }
        Ok(())
    }
    
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
pub mod ffi_helper{
    use std::ffi::CStr;
    use anyhow::Result;

    // 假设每个C字符串后面跟着一个null终止符，并且最后一个指针也是null来表示结束
    pub unsafe fn c_char_arr_to_cstr_arr<'a>(mut camera_ids: *mut *const ::std::os::raw::c_char) -> Result<Vec<&'a CStr>> {
        // 初始化一个临时Vec来存放CStr实例
        let mut cstr_vec: Vec<&CStr> = Vec::new();

        // 遍历C字符串指针数组
        while !camera_ids.is_null() {

            // 解引用指针并创建CStr实例
            let cstr = CStr::from_ptr(*camera_ids);

            // 将CStr加入vec中
            cstr_vec.push(cstr);

            // 移动到下一个C字符串指针
            camera_ids = camera_ids.offset(1);
        }

        Ok(cstr_vec)
    }
}

///全局加载支持中文的字体
pub fn load_global_font(ctx: &egui::Context){
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters):
    fonts.font_data.insert("VonwaonBitmap".to_owned(),
                           egui::FontData::from_static(include_bytes!("../assets/VonwaonBitmap-16px.ttf"))); // .ttf and .otf supported

    // Put my font first (highest priority):
    fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap()
        .insert(0, "VonwaonBitmap".to_owned());

    // Put my font as last fallback for monospace:
    fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap()
        .push("VonwaonBitmap".to_owned());

    ctx.set_fonts(fonts);
}