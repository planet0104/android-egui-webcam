
#[allow(dead_code)]
#[cfg(target_os = "android")]
pub mod permission{
    use anyhow::{anyhow, Result};
    use jni::{objects::{JObject, JString, JValueGen}, sys::{JNIInvokeInterface_, _jobject, jint}, JavaVM};
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
            let granted_int = env.get_static_field("android/content/pm/PackageManager", "PERMISSION_GRANTED", "I")?.i()?;
            // 创建Java字符串
            let permission_str = env.new_string(permission)?;
            let activity: JObject<'_> = JObject::from_raw(app.activity_as_ptr() as *mut _jobject);
            let result = env.call_method(activity, "checkSelfPermission", "(Ljava/lang/String;)I", &[JValueGen::Object(&JObject::from(permission_str))])?.i()?;
            Ok(result == granted_int)
        }
    }

    pub fn get_cache_dir(app: &AndroidApp) -> Result<String>{
        unsafe{
            let vm = JavaVM::from_raw(app.vm_as_ptr() as *mut *const JNIInvokeInterface_)?;
            let mut env = vm.attach_current_thread()?;
            let activity: JObject<'_> = JObject::from_raw(app.activity_as_ptr() as *mut _jobject);

            let file = env.call_method(
                activity,
                "getCacheDir",
                "()Ljava/io/File;",
                &[],
            )?;

            if let JValueGen::Object(file) = file{
                let path = env.call_method(
                    file,
                    "getAbsolutePath",
                    "()Ljava/lang/String;",
                    &[],
                )?;
                
                if let JValueGen::Object(path) = path{
                    let path: JString = path.into();
                    let str = env.get_string(&path)?;
                    let str = std::ffi::CStr::from_ptr(str.get_raw());
                    Ok(str.to_str()?.to_string())
                }else{
                    Err(anyhow!("object is not a string"))
                }
            }else{
                Err(anyhow!("object is not a file"))
            }
        }
    }

    pub fn request_permissions(app: &AndroidApp, permissions:&[&str], request_code: i32) -> Result<()>{
        unsafe{
            let vm = JavaVM::from_raw(app.vm_as_ptr() as *mut *const JNIInvokeInterface_)?;
            let mut env = vm.attach_current_thread()?;
            let activity: JObject<'_> = JObject::from_raw(app.activity_as_ptr() as *mut _jobject);

            // 创建一个Java String数组
            let permission_count = permissions.len() as jint;
            let java_permission_array = env.new_object_array(permission_count, "java/lang/String", JObject::null())?;
            for (index, permission) in permissions.iter().enumerate() {
                let permission_str = env.new_string(*permission)?;
                env.set_object_array_element(&java_permission_array, index as jint, permission_str)?;
            }

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

    pub unsafe fn get_cstr<'a>(s: *const ::std::os::raw::c_char) -> Option<&'a str>{
        let cstr = CStr::from_ptr(s);
        match cstr.to_str(){
            Ok(s) => Some(s),
            Err(err) => None
        }
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


//RGBA顺时针旋转90度
pub fn rotate90(src_buffer: &[u8], new_buffer:&mut [u8], src_width: usize, src_height:usize) -> (usize, usize){
    let (new_width, new_height) = (src_height, src_width);
    for (y, row) in src_buffer.chunks(src_width*4).enumerate(){
        //tx = src_height-y-1;
        //ty = sx;
        let n = (src_height-y-1)*4;
        for (x, pixel) in row.chunks(4).enumerate(){
            let p = x*new_width*4+n;
            new_buffer[p] = pixel[0];
            new_buffer[p+1] = pixel[1];
            new_buffer[p+2] = pixel[2];
            new_buffer[p+3] = pixel[3];
        }
    }
    (new_width, new_height)
}

//RGBA顺时针旋转180度
pub fn rotate180(src_buffer:&[u8], new_buffer:&mut [u8], width: usize, height: usize) -> (usize, usize){
    let stride = width*4;
    let mut p = src_buffer.len()-1;
    for row in src_buffer.chunks(stride){
        for pixel in row.chunks(4){
            new_buffer[p-3] = pixel[0];
            new_buffer[p-2] = pixel[1];
            new_buffer[p-1] = pixel[2];
            new_buffer[p] = pixel[3];
            p -= 4;
        }
    }
    (width, height)
}

//RGBA顺时针旋转270度
pub fn rotate270(src_buffer: &[u8], new_buffer:&mut [u8], src_width: usize, src_height:usize) -> (usize, usize){
    let (new_width, new_height) = (src_height, src_width);
    let src_stride = src_width*4;
    let new_stride = new_width*4;
    for (y, row) in src_buffer.chunks(src_stride).enumerate(){//每一行
        let j = y*4;
        for (x, pixel) in row.chunks(4).enumerate(){//每一列
            let p = (src_width-x-1)*new_stride+j;
            new_buffer[p] = pixel[0];
            new_buffer[p+1] = pixel[1];
            new_buffer[p+2] = pixel[2];
            new_buffer[p+3] = pixel[3];
        }
    }
    (new_width, new_height)
}