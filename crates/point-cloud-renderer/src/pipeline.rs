use nannou::prelude::*;

use crate::{
    camera::{Camera, CameraTransforms},
    point::{CloudData, Point},
};

pub struct GPUPipeline {
    vertex_buffer: wgpu::Buffer,
    vertex_buffer_len: u32,
    initial_vertex_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    cloud_data_buffer: wgpu::Buffer,
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
    render_pipeline: wgpu::RenderPipeline,
    render_bind_group: wgpu::BindGroup,
    compute_bind_group: wgpu::BindGroup,
    compute_pipeline: wgpu::ComputePipeline,
    camera: Camera,
}

impl GPUPipeline {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn new(window: &Window, points: &[Point], camera: Camera, cloud_data: CloudData) -> Self {
        // Initialize utilities
        let device = window.device();
        let msaa_samples = window.msaa_samples();
        let (window_width, window_height) = window.inner_size_pixels();

        // Load shaders
        let compute_shader =
            device.create_shader_module(wgpu::include_wgsl!("shaders/compute.wgsl"));
        let render_shader = device.create_shader_module(wgpu::include_wgsl!("shaders/render.wgsl"));

        // Create the depth buffer texture
        let depth_texture = Self::create_depth_texture(
            device,
            [window_width, window_height],
            Self::DEPTH_FORMAT,
            msaa_samples,
        );
        let depth_texture_view = depth_texture.view().build();

        // Create the initial vertex buffer
        let initial_vertex_buffer = Self::create_initial_vertex_buffer(device, points);

        // Create the vertex buffer
        let vertex_buffer = Self::create_vertex_buffer(device, points);

        // Uniform buffer (for camera)
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniforms Buffer"),
            contents: camera.uniforms().as_bytes(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create the Data storage buffer
        let cloud_data_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Cloud Data Uniforms Buffer"),
            contents: cloud_data.as_bytes(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create the render bind group
        let (render_bind_group_layout, render_bind_group) =
            Self::create_render_bind_group(device, &vertex_buffer, &camera_buffer);

        // Create the compute bind group
        let (compute_bind_group_layout, compute_bind_group) = Self::create_compute_bind_group(
            device,
            &vertex_buffer,
            &initial_vertex_buffer,
            &cloud_data_buffer,
        );

        // Create the pipeline layout
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&render_bind_group_layout],
                push_constant_ranges: &[],
            });
        // Create the render pipeline
        let render_pipeline =
            wgpu::RenderPipelineBuilder::from_layout(&render_pipeline_layout, &render_shader)
                .vertex_entry_point("vs_main")
                .fragment_shader(&render_shader)
                .fragment_entry_point("fs_main")
                .color_format(Frame::TEXTURE_FORMAT)
                .color_blend(wgpu::BlendComponent::REPLACE)
                .alpha_blend(wgpu::BlendComponent::REPLACE)
                .primitive_topology(wgpu::PrimitiveTopology::PointList)
                .depth_format(Self::DEPTH_FORMAT)
                .sample_count(msaa_samples)
                .build(device);

        // Compute pipeline
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "cs_main",
        });

        GPUPipeline {
            vertex_buffer,
            vertex_buffer_len: points.len() as u32,
            initial_vertex_buffer,
            camera_buffer,
            cloud_data_buffer,
            depth_texture,
            depth_texture_view,
            render_bind_group,
            compute_bind_group,
            render_pipeline,
            compute_pipeline,
            camera,
        }
    }

    pub fn render(&mut self, frame: &Frame) {
        let device = frame.device_queue_pair().device();
        let mut encoder = frame.command_encoder();

        // Step 1: Dispatch compute pass
        self.dispatch_compute(&mut encoder);

        // Step 2: Insert buffer barrier to sync compute output to render input
        encoder.insert_debug_marker("Buffer Sync Barrier");

        // If the window has changed size, recreate our depth texture to match.
        if frame.texture_size() != self.depth_texture.size() {
            self.update_depth_texture(device, &mut encoder, frame);
        }

        // Step 3: Dispatch render pass
        self.dispatch_render(&mut encoder, frame);
    }

    pub fn update_camera_transforms(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let camera_storage_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Uniforms Buffer"),
            contents: self.camera.uniforms().as_bytes(),
            usage: wgpu::BufferUsages::COPY_SRC,
        });

        // Copy the new uniforms buffer to the uniform buffer.
        encoder.copy_buffer_to_buffer(
            &camera_storage_buffer,
            0,
            &self.camera_buffer,
            0,
            std::mem::size_of::<CameraTransforms>() as wgpu::BufferAddress,
        );
    }

    pub fn update_cloud_data(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        cloud_data: CloudData,
    ) {
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
            std::mem::size_of::<CloudData>() as wgpu::BufferAddress,
        );
    }

    pub fn new_point_cloud(&mut self, device: &wgpu::Device, points: &[Point]) {
        self.initial_vertex_buffer = Self::create_initial_vertex_buffer(device, points);
        self.vertex_buffer = Self::create_vertex_buffer(device, points);

        // Create the render bind group
        let (_, render_bind_group) =
            Self::create_render_bind_group(device, &self.vertex_buffer, &self.camera_buffer);

        // Create the compute bind group
        let (_, compute_bind_group) = Self::create_compute_bind_group(
            device,
            &self.vertex_buffer,
            &self.initial_vertex_buffer,
            &self.cloud_data_buffer,
        );

        self.render_bind_group = render_bind_group;
        self.compute_bind_group = compute_bind_group;
        self.vertex_buffer_len = points.len() as u32;
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    fn dispatch_compute(&self, encoder: &mut wgpu::CommandEncoder) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute Pass"),
        });
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
        let workgroup_size = 256; // Must match @workgroup_size(256) in the shader
        let num_workgroups = self.vertex_buffer_len.div_ceil(workgroup_size);
        compute_pass.dispatch_workgroups(num_workgroups, 1, 1);
    }

    fn dispatch_render(&self, encoder: &mut wgpu::CommandEncoder, frame: &Frame) {
        let mut render_pass = wgpu::RenderPassBuilder::new()
            .color_attachment(frame.texture_view(), |color| color)
            // We'll use a depth texture to assist with the order of rendering fragments based on depth.
            .depth_stencil_attachment(&self.depth_texture_view, |depth| depth)
            .begin(encoder);
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.render_bind_group, &[]);
        render_pass.draw(0..self.vertex_buffer_len, 0..1);
    }

    fn update_depth_texture(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        frame: &Frame,
    ) {
        self.depth_texture = Self::create_depth_texture(
            device,
            frame.texture_size(),
            self.depth_texture.format(),
            frame.texture_msaa_samples(),
        );
        self.depth_texture_view = self.depth_texture.view().build();
        self.update_camera_transforms(device, encoder);
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

    fn create_vertex_buffer(device: &wgpu::Device, points: &[Point]) -> wgpu::Buffer {
        device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: Point::as_bytes(points),
            usage: wgpu::BufferUsages::STORAGE,
        })
    }

    fn create_initial_vertex_buffer(device: &wgpu::Device, points: &[Point]) -> wgpu::Buffer {
        device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Initial Vertex Buffer"),
            contents: Point::as_bytes(points),
            usage: wgpu::BufferUsages::STORAGE,
        })
    }

    fn create_render_bind_group(
        device: &wgpu::Device,
        vertex_buffer: &wgpu::Buffer,
        camera_buffer: &wgpu::Buffer,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        println!("Creating render bind group");
        let render_bind_group_layout = wgpu::BindGroupLayoutBuilder::new()
            .storage_buffer(wgpu::ShaderStages::VERTEX, false, true)
            .uniform_buffer(wgpu::ShaderStages::VERTEX, false)
            .build(device);
        let render_bind_group = wgpu::BindGroupBuilder::new()
            .binding(vertex_buffer.as_entire_binding())
            .binding(camera_buffer.as_entire_binding())
            .build(device, &render_bind_group_layout);

        println!("Render bind group created");
        (render_bind_group_layout, render_bind_group)
    }

    fn create_compute_bind_group(
        device: &wgpu::Device,
        vertex_buffer: &wgpu::Buffer,
        initial_vertex_buffer: &wgpu::Buffer,
        cloud_data_buffer: &wgpu::Buffer,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        println!("Creating compute bind group");
        let compute_bind_group_layout = wgpu::BindGroupLayoutBuilder::new()
            .storage_buffer(wgpu::ShaderStages::COMPUTE, false, false)
            .storage_buffer(wgpu::ShaderStages::COMPUTE, false, true)
            .uniform_buffer(wgpu::ShaderStages::COMPUTE, false)
            .build(device);
        let compute_bind_group = wgpu::BindGroupBuilder::new()
            .binding(vertex_buffer.as_entire_binding())
            .binding(initial_vertex_buffer.as_entire_binding())
            .binding(cloud_data_buffer.as_entire_binding())
            .build(device, &compute_bind_group_layout);

        println!("Compute bind group created");
        (compute_bind_group_layout, compute_bind_group)
    }
}
