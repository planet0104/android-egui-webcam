use std::ptr::null_mut;
use anyhow::{anyhow, Result};
use log::info;
use ndk_sys::{camera_status_t, ACameraCaptureSession, ACameraCaptureSession_stateCallbacks, ACameraDevice, ACameraDevice_StateCallbacks, ACameraIdList, ACameraManager_create, ACameraManager_getCameraCharacteristics, ACameraManager_getCameraIdList, ACameraMetadata, ACameraOutputTarget, ACaptureRequest, ACaptureSessionOutput, ACaptureSessionOutputContainer};

use crate::utils::ffi_helper;

//参考 https://github.com/justinjoy/native-camera2/blob/master/app/src/main/jni/native-camera2-jni.cpp

pub struct Camera{
    camera_device: *mut ACameraDevice,
    capture_request: *mut ACaptureRequest,
    camera_output_target: *mut ACameraOutputTarget,
    session_output: *mut ACaptureSessionOutput,
    capture_session_output_container: *mut ACaptureSessionOutputContainer,
    capture_session: *mut ACameraCaptureSession,
    device_state_callbacks: *mut ACameraDevice_StateCallbacks,
    capture_session_state_callbacks: *mut ACameraCaptureSession_stateCallbacks,
}

impl Camera{
    pub fn open(&mut self) -> Result<()>{
        unsafe{
            let mut camera_id_list: ACameraIdList = std::mem::zeroed();
            let mut camera_metadata: ACameraMetadata = std::mem::zeroed();

            let camera_manager = ACameraManager_create();

            let camera_status = ACameraManager_getCameraIdList(camera_manager, &mut camera_id_list);
            if camera_status != camera_status_t::ACAMERA_OK {
                return Err(anyhow!("Failed to get camera id list (reason: {:?})", camera_status));
            }
            
            let camera_id_list = &*camera_id_list;

            if camera_id_list.numCameras < 1 {
                return Err(anyhow!("No camera device detected."));
            }

            // selectedCameraId = cameraIdList->cameraIds[0];
            let camera_ids = ffi_helper::c_char_arr_to_cstr_arr(camera_id_list.cameraIds)?;
            let selected_camera_id = camera_ids[0];

            info!("Trying to open Camera2 (id: {:?}, num of camera : {})\n", selected_camera_id.to_str(), camera_ids.len());

            let camera_status = ACameraManager_getCameraCharacteristics(camera_manager, selected_camera_id.as_ptr(), camera_metadata);
        }
        Ok(())
    }
}

impl Drop for Camera {
    fn drop(&mut self) {
        todo!()
    }
}