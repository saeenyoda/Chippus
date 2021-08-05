use crate::emulator::chip8::Emulator;
use crate::emulator::screen::Screen;
use imgui::*;
use imgui_wgpu::{Renderer, Texture, TextureConfig};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BufferUsage, CommandEncoderDescriptor, Device, Extent3d, ImageCopyBuffer, ImageCopyTexture,
    ImageDataLayout, Origin3d, Queue, TextureFormat, TextureUsage,
};

pub struct RGBA {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl RGBA {
    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn to_rgba_normalized(color: [i32; 4]) -> [f32; 4] {
        return [
            (color[0] as f32) / 255.0,
            (color[1] as f32) / 255.0,
            (color[2] as f32) / 255.0,
            (color[3] as f32) / 255.0,
        ];
    }
}

pub struct EmulatorWindow {
    data: Vec<u8>,
    width: usize,
    height: usize,
    scale: f32,
    color: RGBA,
    tex_id: TextureId,
}

impl EmulatorWindow {
    pub fn new(renderer: &mut Renderer, device: &Device) -> EmulatorWindow {
        EmulatorWindow {
            data: vec![0; Screen::WIDTH * Screen::HEIGHT * 4],
            width: Screen::WIDTH,
            height: Screen::HEIGHT,
            scale: 9.0f32,
            color: RGBA {
                r: 0.19f32,
                g: 0.66f32,
                b: 0.38f32,
                a: 1.0f32,
            },
            tex_id: EmulatorWindow::create_texture(
                renderer,
                device,
                Screen::WIDTH as u32,
                Screen::HEIGHT as u32,
            ),
        }
    }

    pub fn render(&mut self, ui: &imgui::Ui) {
        let win = imgui::Window::new(im_str!("Emulator Window")).always_auto_resize(true);
        win.position([5.0f32, 5.0f32], imgui::Condition::Once)
            .build(&ui, || {
                Image::new(
                    self.tex_id,
                    [
                        (self.width as f32) * self.scale,
                        (self.height as f32) * self.scale,
                    ],
                )
                .tint_col(self.color.to_array())
                .build(&ui);

                //ui.input_float(im_str!("Scale"), &mut self.scale).build();
                //ui.same_line(0.0f32);
                imgui::ColorEdit::new(im_str!("Main Color"), &mut self.color.to_array()).build(&ui);
            });
    }

    pub fn update(
        &mut self,
        emulator: &Emulator,
        renderer: &mut Renderer,
        device: &Device,
        mut queue: &mut Queue,
    ) {
        for x in 0..self.width {
            for y in 0..self.height {
                let v = if emulator.screen.get_pixel(x, y) == 1 {
                    255u8
                } else {
                    0
                };

                let pos = (y * 4 * self.width) + (x * 4);
                self.data[pos..pos + 4].copy_from_slice(&[v, v, v, 255u8]);
            }
        }

        // Uploaded updated screen texture data
        self.update_texture(self.tex_id, renderer, &device, &mut queue);
    }

    /// Creates a new wgpu texture made from the imgui font atlas.
    fn create_texture(
        renderer: &mut Renderer,
        device: &Device,
        width: u32,
        height: u32,
    ) -> TextureId {
        // Create the wgpu texture.
        let texture_config = TextureConfig {
            label: None,
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Some(TextureFormat::Rgba8Unorm),
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
        };

        let texture = Texture::new(&device, &renderer, texture_config);

        renderer.textures.insert(texture)
    }

    /// Creates and uploads a new wgpu texture made from the imgui font atlas.
    fn update_texture(
        &mut self,
        id: TextureId,
        renderer: &Renderer,
        device: &Device,
        queue: &mut Queue,
    ) -> Option<bool> {
        // Create the wgpu texture.

        // Upload the actual data to a wgpu buffer.
        let bytes = self.data.len();
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsage::COPY_SRC,
            contents: &self.data[..],
        });

        // Make sure we have an active encoder.
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });

        let img_cl = ImageDataLayout {
            offset: 0,
            bytes_per_row: std::num::NonZeroU32::new(bytes as u32 / self.height as u32),
            rows_per_image: std::num::NonZeroU32::new(self.height as u32),
        };

        // Schedule a copy from the buffer to the texture.
        encoder.copy_buffer_to_texture(
            ImageCopyBuffer {
                buffer: &buffer,
                layout: img_cl,
            },
            ImageCopyTexture {
                texture: &renderer.textures.get(id)?.texture(),
                mip_level: 0,
                origin: Origin3d { x: 0, y: 0, z: 0 },
            },
            Extent3d {
                width: self.width as u32,
                height: self.height as u32,
                depth_or_array_layers: 1,
            },
        );

        // Resolve the actual copy process.
        queue.submit(Some(encoder.finish()));

        Some(true)
    }
}