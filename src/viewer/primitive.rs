use crate::viewer::pipeline::Pipeline;
use crate::viewer::uniforms::Uniforms;
use iced::Rectangle;
use iced::advanced::graphics::Viewport;
use iced::widget::shader::{Storage, wgpu};
use std::sync::Arc;

#[derive(Debug)]
pub struct Primitive {
    pub uniforms: Uniforms,
    pub shader: Arc<String>,
    pub version: usize,
}

impl iced::widget::shader::Primitive for Primitive {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        storage: &mut Storage,
        _bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        let should_store = storage
            .get::<Pipeline>()
            .map(|pipeline| pipeline.version < self.version)
            .unwrap_or(true);

        if should_store {
            storage.store(Pipeline::new(device, format, &self.shader, self.version));
        }

        let pipeline = storage.get_mut::<Pipeline>().unwrap();

        pipeline.prepare(queue, &self.uniforms.to_raw(viewport));
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        storage: &Storage,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        let pipeline = storage.get::<Pipeline>().unwrap();

        pipeline.render(encoder, target, clip_bounds);
    }
}
