use nannou::prelude::*;

use crate::camera::{Camera, CameraConfig, Uniforms};
use crate::point::Point;

pub struct GPUPipeline {
    vertex_buffer: wgpu::Buffer,
    vertex_buffer_len: u32,
    uniform_buffer: wgpu::Buffer,
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    camera: Camera,
    camera_config: CameraConfig,
}

impl GPUPipeline {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn new(window: &Window, points: &[Point], camera: Camera) -> Self {
        let device = window.device();
        let msaa_samples = window.msaa_samples();
        let (window_width, window_height) = window.inner_size_pixels();

        let shader_mod = device.create_shader_module(wgpu::include_wgsl!("shaders/cloud.wgsl"));

        // Create the vertex buffer
        let vertices_bytes = Point::as_bytes(points);
        let vertex_buffer_len = points.len() as u32;
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: vertices_bytes,
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create the depth buffer texture
        let depth_texture = create_depth_texture(
            device,
            [window_width, window_height],
            Self::DEPTH_FORMAT,
            msaa_samples,
        );
        let depth_texture_view = depth_texture.view().build();

        // Create the uniform buffer (camera transforms)
        let camera_config = CameraConfig::default().with_aspect_ratio(window_width, window_height);
        let uniforms = camera_config.uniform(camera.view());
        let uniforms_bytes = uniforms.as_bytes();
        let uniforms_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: uniforms_bytes,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create the uniforms bind group
        let uniforms_bind_group_layout = wgpu::BindGroupLayoutBuilder::new()
            .uniform_buffer(wgpu::ShaderStages::VERTEX, false)
            .build(device);
        let uniforms_bind_group = wgpu::BindGroupBuilder::new()
            .buffer::<Uniforms>(&uniforms_buffer, 0..1)
            .build(device, &uniforms_bind_group_layout);

        // Create the pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniforms_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create the render pipeline
        let render_pipeline =
            wgpu::RenderPipelineBuilder::from_layout(&pipeline_layout, &shader_mod)
                .vertex_entry_point("vs_main")
                .fragment_shader(&shader_mod)
                .fragment_entry_point("fs_main")
                .color_format(Frame::TEXTURE_FORMAT)
                .color_blend(wgpu::BlendComponent::REPLACE)
                .alpha_blend(wgpu::BlendComponent::REPLACE)
                .primitive_topology(wgpu::PrimitiveTopology::PointList)
                .add_vertex_buffer::<Point>(&Point::ATTRIBS)
                .depth_format(Self::DEPTH_FORMAT)
                .sample_count(msaa_samples)
                .build(device);

        GPUPipeline {
            vertex_buffer,
            vertex_buffer_len,
            uniform_buffer: uniforms_buffer,
            depth_texture,
            depth_texture_view,
            bind_group: uniforms_bind_group,
            render_pipeline,
            camera,
            camera_config,
        }
    }

    pub fn render(&mut self, frame: &Frame) {
        let device = frame.device_queue_pair().device();
        let mut encoder = frame.command_encoder();

        // If the window has changed size, recreate our depth texture to match.
        let depth_size = self.depth_texture.size();
        let frame_size = frame.texture_size();
        if frame_size != depth_size {
            let depth_format = self.depth_texture.format();
            let sample_count = frame.texture_msaa_samples();
            self.depth_texture =
                create_depth_texture(device, frame_size, depth_format, sample_count);
            self.depth_texture_view = self.depth_texture.view().build();
            self.update_uniforms(device, &mut encoder);
        }

        // Record commands for rendering the frame.
        let mut render_pass = wgpu::RenderPassBuilder::new()
            .color_attachment(frame.texture_view(), |color| color)
            // We'll use a depth texture to assist with the order of rendering fragments based on depth.
            .depth_stencil_attachment(&self.depth_texture_view, |depth| depth)
            .begin(&mut encoder);
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..self.vertex_buffer_len, 0..1);
    }

    pub fn update_uniforms(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        // Update the uniforms
        let uniforms = self.camera_config.uniform(self.camera.view());
        let uniforms_size = std::mem::size_of::<Uniforms>() as wgpu::BufferAddress;
        let new_uniforms_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: uniforms.as_bytes(),
            usage: wgpu::BufferUsages::COPY_SRC,
        });

        // Copy the new uniforms buffer to the uniform buffer.
        encoder.copy_buffer_to_buffer(
            &new_uniforms_buffer,
            0,
            &self.uniform_buffer,
            0,
            uniforms_size,
        );
    }

    pub fn update_points(&mut self, device: &wgpu::Device, points: &[Point]) {
        // Update the vertex buffer with the new points.
        let vertices_bytes = Point::as_bytes(points);
        let vertex_buffer_len = points.len() as u32;
        let new_vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: vertices_bytes,
            usage: wgpu::BufferUsages::VERTEX,
        });

        self.vertex_buffer = new_vertex_buffer;
        self.vertex_buffer_len = vertex_buffer_len;
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }
}

fn create_depth_texture(
    device: &wgpu::Device,
    size: [u32; 2],
    depth_format: wgpu::TextureFormat,
    sample_count: u32,
) -> wgpu::Texture {
    wgpu::TextureBuilder::new()
        .size(size)
        .format(depth_format)
        .usage(wgpu::TextureUsages::RENDER_ATTACHMENT)
        .sample_count(sample_count)
        .build(device)
}
