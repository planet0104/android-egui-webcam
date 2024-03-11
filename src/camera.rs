//参考 https://github.com/justinjoy/native-camera2/blob/master/app/src/main/jni/native-camera2-jni.cpp
//https://github.com/qi-xmu/Android-ndk-camera-zh/blob/b5b3c802c4ecb997e043f166f316eb2c7f3b3c54/app/src/main/cpp/camera_engine.cpp#L114
//https://github.com/RMerl/asuswrt-merlin.ng/blob/26f484e9427f15675afb228718ca2ee71336856e/release/src/router/ffmpeg/libavdevice/android_camera.c#L229

use core::slice;
use std::{ffi::{c_int, c_void, CStr}, mem::zeroed, ptr::null_mut};
use anyhow::{anyhow, Result};
use image::RgbaImage;
use log::{error, info, warn};
use ndk_sys::{acamera_metadata_tag, camera_status_t, media_status_t, ACameraCaptureSession, ACameraCaptureSession_setRepeatingRequest, ACameraCaptureSession_stateCallbacks, ACameraDevice, ACameraDevice_StateCallbacks, ACameraDevice_close, ACameraDevice_createCaptureRequest, ACameraDevice_createCaptureSession, ACameraDevice_getId, ACameraDevice_request_template, ACameraIdList, ACameraManager_create, ACameraManager_delete, ACameraManager_deleteCameraIdList, ACameraManager_getCameraCharacteristics, ACameraManager_getCameraIdList, ACameraManager_openCamera, ACameraMetadata, ACameraMetadata_const_entry, ACameraMetadata_free, ACameraMetadata_getConstEntry, ACameraOutputTarget, ACameraOutputTarget_create, ACameraOutputTarget_free, ACaptureRequest, ACaptureRequest_addTarget, ACaptureRequest_free, ACaptureSessionOutput, ACaptureSessionOutputContainer, ACaptureSessionOutputContainer_add, ACaptureSessionOutputContainer_create, ACaptureSessionOutputContainer_free, ACaptureSessionOutput_create, ACaptureSessionOutput_free, AImageReader, AImageReader_ImageListener, AImageReader_acquireLatestImage, AImageReader_getWindow, AImageReader_new, AImageReader_setImageListener, AImage_delete, AImage_getPlaneData, AImage_getPlaneRowStride, AImage_getTimestamp, ANativeWindow, AIMAGE_FORMATS};
use winit::platform::android::activity::AndroidApp;

use crate::utils::{ffi_helper, permission::get_cache_dir, yuv_to_rgba};

#[link(name = "camera2ndk")] extern "C" {}

#[link(name = "mediandk")] extern "C" {}

pub struct Camera{
    app: AndroidApp,
    camera_device: *mut ACameraDevice,
    capture_request: *mut ACaptureRequest,
    camera_output_target: *mut ACameraOutputTarget,
    session_output: *mut ACaptureSessionOutput,
    capture_session_output_container: *mut ACaptureSessionOutputContainer,
    image_reader: *mut AImageReader,
    /// width,height,format
    image_formats: Vec<(i32, i32, i32)>,
    camera_id: Option<String>,
    image_listener: AImageReader_ImageListener,
    capture_session_state_callbacks: ACameraCaptureSession_stateCallbacks,
    device_state_callbacks: ACameraDevice_StateCallbacks,
    preview_width: i32,
    preview_height: i32,
}

impl Camera{
    pub fn new(app: AndroidApp) -> Self {
        Self { camera_device: null_mut(), capture_request: null_mut(), camera_output_target: null_mut(), session_output: null_mut(), capture_session_output_container: null_mut(), image_reader: null_mut(), image_formats: vec![], camera_id: None, image_listener: AImageReader_ImageListener{
            context: null_mut(),
            onImageAvailable: None,
        },
        capture_session_state_callbacks: unsafe{ zeroed() },
        device_state_callbacks: unsafe{ zeroed()},
        app,
        preview_width: 0,
        preview_height: 0,
        }
    }

    pub fn open(&mut self, camera_id: &str) -> Result<()>{
        unsafe{
            info!("open camera 001..");
            let camera_manager = ACameraManager_create();
            info!("open camera 002..");
            let mut camera_id_list_raw = null_mut();
            info!("open camera 003..");
            let camera_status = ACameraManager_getCameraIdList(camera_manager, &mut camera_id_list_raw);
            info!("open camera 004..");
            if camera_status != camera_status_t::ACAMERA_OK {
                return Err(anyhow!("Failed to get camera id list (reason: {:?})", camera_status));
            }
            info!("camera_id_list_raw={:?}", camera_id_list_raw);

            if camera_id_list_raw.is_null(){
                return Err(anyhow!("Failed to get camera id list (reason: camera_id_list is null)"));
            }

            let camera_id_list = &*camera_id_list_raw;

            info!("camera_id_list.cameraIds {:?}", camera_id_list.cameraIds);
            info!("camera_id_list.cameraIds.is_null = {}", camera_id_list.cameraIds.is_null());

            if camera_id_list.numCameras < 1 {
                return Err(anyhow!("No camera device detected."));
            }

            info!("open camera 007..");

            let camera_ids = slice::from_raw_parts(camera_id_list.cameraIds, camera_id_list.numCameras as usize);

            info!("open camera 008..");

            let camera_id_strings:Vec<String> = camera_ids.iter().map(|v| CStr::from_ptr(*v).to_str().unwrap_or("").to_string()).collect();
            info!("camera_ids: {:?}", camera_id_strings);

            let mut selected_camera_id = None;
            let mut selected_camera_idx = -1;
            for (idx, cid) in camera_ids.iter().enumerate(){
                if CStr::from_ptr(*cid).to_str().unwrap_or("-1") == camera_id{
                    selected_camera_id = Some(cid);
                    selected_camera_idx = idx as i32;
                    break;
                }
            }
            
            if selected_camera_id.is_none(){
                return Err(anyhow!("Camera Id not found."));
            }
            let selected_camera_id = selected_camera_id.unwrap();

            info!("Trying to open Camera2 (index: {:?}, num of camera : {})", selected_camera_idx, camera_id_strings.len());

            let mut camera_metadata = null_mut();

            let camera_status = ACameraManager_getCameraCharacteristics(camera_manager, *selected_camera_id, &mut camera_metadata);
            if camera_status != camera_status_t::ACAMERA_OK {
                return Err(anyhow!("Failed to get camera meta data of index:{:?}", selected_camera_idx));
            }

            info!("camera_metadata is null? {}", camera_metadata.is_null());

            let (lens_facing, sensor_orientation) = Camera::get_sensor_orientation(camera_metadata);
            info!("lens_facing: {lens_facing}");
            info!("sensor_orientation: {sensor_orientation}");

            // 获取相机支持的分辨率
            self.image_formats = Camera::get_video_size(camera_metadata)?;

            info!("image_formats: {:?}", self.image_formats);

            unsafe extern "C" fn on_disconnected(_data: *mut c_void, device: *mut ACameraDevice) {
                info!("Camera(id: {:?}) is disconnected.", ffi_helper::get_cstr(ACameraDevice_getId(device)));
            }

            unsafe extern "C" fn on_error(_data: *mut c_void, device: *mut ACameraDevice, error: c_int) {
                error!("Error(code: {}) on Camera(id: {:?}).", error, ffi_helper::get_cstr(ACameraDevice_getId(device)));
            }

            self.device_state_callbacks.onDisconnected = Some(on_disconnected);
            self.device_state_callbacks.onError = Some(on_error);

            let camera_status = ACameraManager_openCamera(camera_manager, *selected_camera_id, &mut self.device_state_callbacks, &mut self.camera_device);

            if camera_status != camera_status_t::ACAMERA_OK {
                return Err(anyhow!("Failed to open camera device (index: {})", selected_camera_idx));
            }

            self.camera_id = Some(camera_id.to_string());

            ACameraMetadata_free(camera_metadata);
            ACameraManager_deleteCameraIdList(camera_id_list_raw);
            ACameraManager_delete(camera_manager);
        }
        Ok(())
    }

    fn get_sensor_orientation(camera_metadata: *mut ACameraMetadata) -> (u8, i32){
        unsafe{
            let mut lens_facing: ACameraMetadata_const_entry = zeroed();
            let mut sensor_orientation: ACameraMetadata_const_entry  = zeroed();

            ACameraMetadata_getConstEntry(camera_metadata, acamera_metadata_tag::ACAMERA_LENS_FACING.0, &mut lens_facing);
            ACameraMetadata_getConstEntry(camera_metadata,acamera_metadata_tag::ACAMERA_SENSOR_ORIENTATION.0, &mut sensor_orientation);

            let u8_arr = slice::from_raw_parts(lens_facing.data.u8_, lens_facing.count as usize);
            let i32_arr = slice::from_raw_parts(lens_facing.data.i32_, lens_facing.count as usize);
            let lens_facing = u8_arr[0];
            let sensor_orientation = i32_arr[0];
            (lens_facing, sensor_orientation)
        }
    }

    // 获取相机支持的分辨率
    fn get_video_size(camera_metadata: *mut ACameraMetadata) -> Result<Vec<(i32, i32, i32)>>{
        unsafe{
            let mut available_configs:ACameraMetadata_const_entry = zeroed();
            let camera_status = ACameraMetadata_getConstEntry(camera_metadata, acamera_metadata_tag::ACAMERA_SCALER_AVAILABLE_STREAM_CONFIGURATIONS.0, &mut available_configs);
            if camera_status != camera_status_t::ACAMERA_OK {
                return Err(anyhow!("Failed to get ACameraMetadata_const_entry res={:?}", camera_status));
            }
            
            // 数据格式: format, width, height, input?, type int32
            let mut formats: Vec<(i32, i32, i32)> = vec![];
            let data_i32_list:&[i32] =  slice::from_raw_parts(available_configs.data.i32_, available_configs.count as usize);
            for i in 0..available_configs.count as usize{
                let input_idx = i*4+3;
                let format_idx = i*4+0;
                if format_idx >= available_configs.count as usize{
                    break;
                }
                let input = data_i32_list[input_idx];
                let format = data_i32_list[format_idx];
    
                if input != 0 {
                    continue;
                }
                
                if format == AIMAGE_FORMATS::AIMAGE_FORMAT_YUV_420_888.0 as i32{
                    let width = data_i32_list[i*4 + 1];
                    let height = data_i32_list[i*4 + 2];
                    info!("YUV_420: {width}x{height}");
                    formats.push((width, height, format));
                }
            }
            Ok(formats)
        }
    }
    
    pub fn close(&mut self){
        unsafe{
            if !self.image_reader.is_null() {
                ACaptureRequest_free(self.capture_request);
                self.capture_request = null_mut();
            }
    
            if !self.camera_output_target.is_null(){
                ACameraOutputTarget_free(self.camera_output_target);
                self.camera_output_target = null_mut();
            }
    
            if !self.camera_device.is_null() {
                let camera_status = ACameraDevice_close(self.camera_device);
                
                if camera_status != camera_status_t::ACAMERA_OK {
                    error!("Failed to close CameraDevice.");
                }
                self.camera_device = null_mut();
            }
    
            if !self.session_output.is_null() {
                ACaptureSessionOutput_free(self.session_output);
                self.session_output = null_mut();
            }
    
            if !self.capture_session_output_container.is_null() {
                ACaptureSessionOutputContainer_free(self.capture_session_output_container);
                self.capture_session_output_container = null_mut();
            }
        }
        info!("Close Camera");
    }

    pub fn start_preview(&mut self, width: i32, height: i32) -> Result<()>{
        self.preview_width = width;
        self.preview_height = height;
        self.create_image_reader(width, height, AIMAGE_FORMATS::AIMAGE_FORMAT_YUV_420_888)?;
        unsafe{
            let camera_status = ACameraDevice_createCaptureRequest(self.camera_device, ACameraDevice_request_template::TEMPLATE_PREVIEW, &mut self.capture_request);

            if camera_status != camera_status_t::ACAMERA_OK {
                return Err(anyhow!("Failed to create preview capture request (id: {:?})", self.camera_id));
            }

            let mut native_window: *mut ANativeWindow = null_mut();
            let res = AImageReader_getWindow(self.image_reader, &mut native_window);

            if res != media_status_t::AMEDIA_OK {
                error!("AImageReader_getWindow error.");
            }

            ACameraOutputTarget_create(native_window, &mut self.camera_output_target);
            ACaptureRequest_addTarget(self.capture_request, self.camera_output_target);

            let mut session_output = null_mut();
            ACaptureSessionOutput_create(native_window, &mut session_output);

            let camera_status = ACaptureSessionOutputContainer_create(&mut self.capture_session_output_container);

            if camera_status != camera_status_t::ACAMERA_OK {
                return Err(anyhow!("Failed to create capture session output container (reason: {:?})", camera_status));
            }

            unsafe extern "C" fn capture_session_on_ready(context: *mut c_void, session: *mut ACameraCaptureSession) {
                info!("Session is ready. {:?}", session);
                let camera_ptr:*mut Camera = context as *mut _ as *mut Camera;
            }
            
            unsafe extern "C" fn capture_session_on_active(context: *mut c_void, session: *mut ACameraCaptureSession) {
                info!("Session is activated. {:?}", session);
                let camera_ptr:*mut Camera = context as *mut _ as *mut Camera;
            }
            
            unsafe extern "C" fn capture_session_on_closed(context: *mut c_void, session: *mut ACameraCaptureSession) {
                info!("Session is closed. {:?}", session);
                let camera_ptr:*mut Camera = context as *mut _ as *mut Camera;
            }

            self.capture_session_state_callbacks.onReady = Some(capture_session_on_ready);
            self.capture_session_state_callbacks.onActive = Some(capture_session_on_active);
            self.capture_session_state_callbacks.onClosed = Some(capture_session_on_closed);
            self.capture_session_state_callbacks.context = (self as *mut _) as *mut c_void;

            ACaptureSessionOutputContainer_add(self.capture_session_output_container, session_output);
            
            let mut capture_session = null_mut();
            let camera_status = ACameraDevice_createCaptureSession(self.camera_device, self.capture_session_output_container,
                                               &self.capture_session_state_callbacks, &mut capture_session);

            if camera_status != camera_status_t::ACAMERA_OK {
                return Err(anyhow!("Failed to create capture session (reason: {:?})", camera_status));
            }

            let camera_status = ACameraCaptureSession_setRepeatingRequest(capture_session, null_mut(), 1, &mut self.capture_request, null_mut());

            if camera_status != camera_status_t::ACAMERA_OK {
                return Err(anyhow!("Failed to set repeating request (reason: {:?})", camera_status));
            }
        }
        Ok(())
    }

    fn on_image_available(&mut self){
        unsafe{
            let mut image = null_mut();
            let media_status = AImageReader_acquireLatestImage(self.image_reader, &mut image);
            if media_status != media_status_t::AMEDIA_OK {
                if media_status == media_status_t::AMEDIA_IMGREADER_NO_BUFFER_AVAILABLE {
                    warn!("An image reader frame was discarded");
                } else {
                    error!("Failed to acquire latest image from image reader, error: {:?}.", media_status);
                }
            }else{
                let mut timestamp_ns = 0;
                let mut row_stride: [i32; 4] = [0, 0, 0, 0];
                let mut plane_data_length: [i32; 4] = [0, 0, 0, 0];
                let mut image_plane_data: [*mut u8; 4] = zeroed();

                let _ = AImage_getTimestamp(image, &mut timestamp_ns);

                let _ = AImage_getPlaneRowStride(image, 0, &mut row_stride[0]);

                let _ = AImage_getPlaneData(image, 0, &mut image_plane_data[0], &mut plane_data_length[0]);


                let _ = AImage_getPlaneRowStride(image, 1, &mut row_stride[1]);
                let _ = AImage_getPlaneData(image, 1, &mut image_plane_data[1], &mut plane_data_length[1]);
                let _ = AImage_getPlaneRowStride(image, 2, &mut row_stride[2]);
                let _ = AImage_getPlaneData(image, 2, &mut image_plane_data[2], &mut plane_data_length[2]);

                info!("获取到了预览帧 timestamp_ns={timestamp_ns}");

                let plan_slice_0 = slice::from_raw_parts(image_plane_data[0], row_stride[0] as usize * self.preview_height as usize);
                let plan_slice_1 = slice::from_raw_parts(image_plane_data[1], row_stride[1] as usize * self.preview_height as usize);
                let plan_slice_2 = slice::from_raw_parts(image_plane_data[2], row_stride[2] as usize * self.preview_height as usize);
                
                match yuv_to_rgba(self.preview_width as usize, self.preview_height as usize, plan_slice_0, plan_slice_1, plan_slice_2){
                    Ok(rgba_data) => {
                        match get_cache_dir(&self.app){
                            Ok(cache_dir) => {
                                let img = RgbaImage::from_raw(self.preview_width as u32, self.preview_height as u32, rgba_data).unwrap();
                                let yuv_path = format!("{}/{}.jpg", cache_dir, timestamp_ns);
                                img.save(&yuv_path).unwrap();
                                info!("临时文件写入成功:{yuv_path}");
                            },
                            Err(err) => error!("临时文件夹获取失败:{:?}", err)
                        };
                    }
                    Err(err) => {
                        error!("yuv转换失败:{:?}", err);
                    }
                }
            }

            AImage_delete(image);
        }
    }
    
    fn create_image_reader(&mut self, width: i32, height: i32, image_format: AIMAGE_FORMATS) -> Result<()>{
        unsafe{
            let res: ndk_sys::media_status_t = AImageReader_new(width, height, image_format.0 as i32, 2, &mut self.image_reader);

            if res != media_status_t::AMEDIA_OK {
                return Err(anyhow!("create Image Reader error."));
            }

            unsafe extern "C" fn on_image_available(context: *mut c_void, image_reader: *mut AImageReader){
                //还原Camera指针
                let camera = &mut *(context as *mut _ as *mut Camera);
                camera.on_image_available();
            }

            let camera_ptr: *mut Camera = self as *mut _;

            self.image_listener.context = camera_ptr as *mut c_void;
            self.image_listener.onImageAvailable = Some(on_image_available);

            let res = AImageReader_setImageListener(self.image_reader, &mut self.image_listener);
            if res != media_status_t::AMEDIA_OK {
                return Err(anyhow!("set Image Listener error."));
            }
        }
        Ok(())
    }
}

impl Drop for Camera{
    fn drop(&mut self) {
        let _ = self.close();
    }
}