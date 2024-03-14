use std::sync::mpsc::{channel, Receiver};
use egui::{vec2, Button, ImageData, Rect, TextureHandle, TextureOptions, Vec2};
use log::info;
use winit::platform::android::activity::AndroidApp;

use crate::{camera::Camera, utils};

pub struct App {
    #[cfg(target_os = "android")]
    app: AndroidApp,
    camera: Camera,
    frame_texture: Option<TextureHandle>,
    image_receiver: Receiver<ImageData>,
}

impl App {
    pub fn new(
        #[cfg(target_os = "android")]
        app: AndroidApp
    ) -> Self{

        let (image_sender, image_receiver) = channel();
        Self{
            frame_texture: None,
            #[cfg(target_os = "android")]
            app: app.clone(),
            #[cfg(target_os = "android")]
            camera: Camera::new(app.clone(), image_sender),
            image_receiver
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        egui::Window::new("Camera")
            .show(ctx, |ui| {

                if self.frame_texture.is_none(){
                    self.frame_texture.replace(ui.ctx().load_texture(
                        "my-image",
                        egui::ColorImage::example(),
                        Default::default()
                    ));
                }

                let frame_texture = self.frame_texture.as_mut().unwrap();

                if let Ok(buf) = self.image_receiver.try_recv(){
                    frame_texture.set(buf, TextureOptions::LINEAR);
                }

                //1280x960
                //426x320
                ui.image(frame_texture.id(), [320., 426.]);
                
                ui.label("hello! hello!");
                if ui.add(Button::new("申请相机权限").min_size(Vec2::new(100., 50.))).clicked(){
                    #[cfg(target_os = "android")]
                    {
                        info!("申请权限。。。");
                        let res = utils::permission::request_camera_permission(&self.app);
                        info!("权限申请结果:{:?}", res);
                    }
                }
                if ui.add(Button::new("打开相机").min_size(Vec2::new(100., 50.))).clicked(){
                    #[cfg(target_os = "android")]
                    {
                        info!("打开相机...");
                        let res = self.camera.open("0");
                        info!("打开相机:{:?}", res);
                        let res = self.camera.start_preview(1280, 960);
                        info!("预览相机:{:?}", res);
                    }   
                }
                if ui.add(Button::new("关闭相机").min_size(Vec2::new(100., 50.))).clicked(){
                    #[cfg(target_os = "android")]
                    {
                        info!("关闭相机...");
                        self.camera.close();
                    }   
                }
            });
    }
}