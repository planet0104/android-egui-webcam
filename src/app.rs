use std::{sync::{Arc, RwLock}, time::Instant};

use egui::{ColorImage, TextureOptions};
use log::info;
use winit::platform::android::activity::AndroidApp;

use crate::utils;

pub struct App {
    image_buffer: Arc<RwLock<ColorImage>>,
    texture: Option<egui::TextureHandle>,
    #[cfg(target_os = "android")]
    app: AndroidApp,
}

impl App {
    pub fn new(
        #[cfg(target_os = "android")]
        app: AndroidApp
    ) -> Self{
        Self{
            image_buffer: Arc::new(RwLock::new(ColorImage::default())),
            texture: None,
            #[cfg(target_os = "android")]
            app
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        egui::Window::new("app")
            .default_width(320.0)
            .show(ctx, |ui| {
                // let texture: &mut egui::TextureHandle = self.texture.get_or_insert_with(|| {
                //     // Load the texture only once.
                //     ui.ctx().load_texture(
                //         "my-image",
                //         egui::ColorImage::example(),
                //         Default::default()
                //     )
                // });
    
                // let t1 = Instant::now();
                // if let Ok(buf) = self.image_buffer.try_read(){
                //     texture.set(buf.clone(), TextureOptions::LINEAR);
                // }
                // println!("耗时:{}ms", t1.elapsed().as_millis());
                
                ui.label("hello! hello!");
                if ui.button("申请相机权限").clicked(){
                    #[cfg(target_os = "android")]
                    {
                        info!("申请权限。。。");
                        let res = utils::permission::request_camera_permission(&self.app);
                        info!("权限申请结果:{:?}", res);
                    }
                }
                if ui.button("打开相机").clicked(){
                    #[cfg(target_os = "android")]
                    {
                        info!("打开相机...");
                    }   
                }
                // ui.image((texture.id(), texture.size_vec2()));
            });
    }
}