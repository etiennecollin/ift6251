use nannou::prelude::*;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::camera::{Camera, CameraConfig, CameraTransforms};
use crate::point::{CloudData, Point};

pub struct GPUPipeline {
    number_points: u32,
    vertex_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    cloud_data_buffer: wgpu::Buffer,
    current_positions_buffer: wgpu::Buffer,
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    camera: Camera,
    camera_config: CameraConfig,
}

impl GPUPipeline {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn new(window: &Window, points: &[Point], cloud_data: CloudData, camera: Camera) -> Self {
        let device = window.device();
        let msaa_samples = window.msaa_samples();
        let (window_width, window_height) = window.inner_size_pixels();

        let shader_mod = device.create_shader_module(wgpu::include_wgsl!("shaders/cloud.wgsl"));

        // Create the depth buffer texture
        let depth_texture = create_depth_texture(
            device,
            [window_width, window_height],
            Self::DEPTH_FORMAT,
            msaa_samples,
        );
        let depth_texture_view = depth_texture.view().build();

        // Create the vertex buffer
        let (vertex_buffer, number_points) = create_vertex_buffer(device, points);

        // Create the camera transforms buffer
        let camera_config = CameraConfig::default().with_aspect_ratio(window_width, window_height);
        let camera_uniforms = camera_config.uniforms(camera.view());
        let camera_uniforms_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Uniforms Buffer"),
            contents: camera_uniforms.as_bytes(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create the Data storage buffer
        let cloud_data_storage_buffer = create_cloud_data_buffer(device, cloud_data);
        let (current_positions_storage_buffer, current_positions_storage_size) =
            create_current_positions_buffer(device, points);

        // Create the uniforms bind group
        let bind_group_layout = wgpu::BindGroupLayoutBuilder::new()
            .uniform_buffer(wgpu::ShaderStages::VERTEX, false)
            .uniform_buffer(wgpu::ShaderStages::VERTEX, false)
            .storage_buffer(wgpu::ShaderStages::VERTEX, false, true) // TODO: Set readonly to false
            .build(device);
        let bind_group = wgpu::BindGroupBuilder::new()
            .buffer::<CameraTransforms>(&camera_uniforms_buffer, 0..1)
            .buffer::<CloudData>(&cloud_data_storage_buffer, 1..2)
            .buffer_bytes(
                &current_positions_storage_buffer,
                0,
                Some(current_positions_storage_size),
            )
            .build(device, &bind_group_layout);

        // Create the pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
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
            number_points,
            vertex_buffer,
            camera_buffer: camera_uniforms_buffer,
            cloud_data_buffer: cloud_data_storage_buffer,
            current_positions_buffer: current_positions_storage_buffer,
            depth_texture,
            depth_texture_view,
            bind_group,
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
            self.update_camera_transforms(device, &mut encoder);
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
        render_pass.draw(0..self.number_points, 0..1);
    }

    pub fn update_camera_transforms(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let camera_storage = self.camera_config.uniforms(self.camera.view());
        let camera_storage_size = std::mem::size_of::<CameraTransforms>() as wgpu::BufferAddress;
        let camera_storage_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Uniforms Buffer"),
            contents: camera_storage.as_bytes(),
            usage: wgpu::BufferUsages::COPY_SRC,
        });

        // Copy the new uniforms buffer to the uniform buffer.
        encoder.copy_buffer_to_buffer(
            &camera_storage_buffer,
            0,
            &self.camera_buffer,
            0,
            camera_storage_size,
        );
    }

    pub fn update_cloud_data(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        cloud_data: CloudData,
    ) {
        let cloud_data_size = std::mem::size_of::<CloudData>() as wgpu::BufferAddress;
        let cloud_data_storage_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Cloud Data Uniforms Buffer"),
            contents: cloud_data.as_bytes(),
            usage: wgpu::BufferUsages::COPY_SRC,
        });

        // Copy the new uniforms buffer to the uniform buffer.
        encoder.copy_buffer_to_buffer(
            &cloud_data_storage_buffer,
            0,
            &self.cloud_data_buffer,
            0,
            cloud_data_size,
        );
    }

    pub fn new_cloud(&mut self, device: &wgpu::Device, points: &[Point]) {
        let (vertex_buffer, number_points) = create_vertex_buffer(device, points);
        let (current_positions_storage_buffer, _) = create_current_positions_buffer(device, points);

        self.vertex_buffer = vertex_buffer;
        self.current_positions_buffer = current_positions_storage_buffer;
        self.number_points = number_points;
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }
}

fn create_current_positions_buffer(
    device: &wgpu::Device,
    points: &[Point],
) -> (wgpu::Buffer, wgpu::BufferSize) {
    let current_positions = points
        .par_iter()
        .map(|point| point.position)
        .collect::<Vec<[f32; 3]>>();
    let current_positions_bytes = unsafe { wgpu::bytes::from_slice(&current_positions) };
    let current_positions_size =
        wgpu::BufferSize::new((points.len() * std::mem::size_of::<[f32; 3]>()) as u64).unwrap();
    let current_positions_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Current Positions Buffer"),
        contents: current_positions_bytes,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    });

    (current_positions_buffer, current_positions_size)
}

fn create_vertex_buffer(device: &wgpu::Device, points: &[Point]) -> (wgpu::Buffer, u32) {
    // Create the vertex buffer
    let vertices_bytes = Point::as_bytes(points);
    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: vertices_bytes,
        usage: wgpu::BufferUsages::VERTEX,
    });
    (vertex_buffer, points.len() as u32)
}

fn create_cloud_data_buffer(device: &wgpu::Device, cloud_data: CloudData) -> wgpu::Buffer {
    let cloud_data_bytes = cloud_data.as_bytes();
    device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Cloud Data Uniforms Buffer"),
        contents: cloud_data_bytes,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
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
