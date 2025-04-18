use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

use ift6251::get_save_path;
use nannou::{prelude::*, state::keys, winit};
use nannou_audio::{Buffer, Host, Stream};
use nannou_egui::{
    Egui,
    egui::{self},
};
use point_cloud_renderer::{
    camera::{Camera, CameraConfig, Direction},
    loader::{generate_random_point_cloud, read_e57},
    pipeline::GPUPipeline,
    point::{CloudData, Point},
};
use spectrum_analyzer::{FrequencyLimit, samples_fft_to_spectrum, windows::hann_window};

fn main() {
    nannou::app(model).event(event).update(update).run();
}

struct State {
    cloud_file_path: String,
    audio_file_path: String,
    movement_speed: f64,
    mouse_sensitivity: f32,
    cloud_data: CloudData,
    // This will be accessed by the audio thread.
    fft_output: Arc<Mutex<f32>>,
}

struct Audio {
    sounds: Vec<audrey::read::BufFileReader>,
    fft_output: Arc<Mutex<f32>>,
}

struct Model {
    window_id: WindowId,
    egui: Egui,
    state: State,
    audio_stream: Stream<Audio>,
    shader_pipeline: RefCell<GPUPipeline>,
    update_camera: RefCell<bool>,
    update_cloud_data: RefCell<bool>,
    camera_is_active: bool,
}

fn random_points() -> Vec<Point> {
    let range_x = (-100.0, 100.0);
    let range_y = (-100.0, 100.0);
    let range_z = (-100.0, 100.0);
    generate_random_point_cloud(5000000, range_x, range_y, range_z)
}

fn model(app: &App) -> Model {
    // Setup app
    app.set_fullscreen_on_shortcut(true);

    // Set GPU device descriptor
    let descriptor = wgpu::DeviceDescriptor {
        label: Some("Point Cloud Renderer Device"),
        features: wgpu::Features::default(),
        limits: wgpu::Limits {
            max_storage_buffer_binding_size: 2 << 30, // To support big point clouds
            // max_texture_dimension_2d: 2 << 14,        // To support the big 9x3 4K display wall
            ..Default::default()
        },
    };

    // Create a new window
    let window_id = app
        .new_window()
        .fullscreen()
        .view(view)
        .raw_event(raw_window_event)
        .key_pressed(key_pressed)
        .device_descriptor(descriptor)
        .build()
        .unwrap();
    let window = app.window(window_id).unwrap();
    let (window_width, window_height) = window.inner_size_pixels();

    // Get control of the cursor
    let camera_is_active = true;
    if let Err(e) = window.set_cursor_grab(true) {
        eprintln!("warning: cursor grabbing not supported: {e}");
    }
    window.set_cursor_visible(false);

    // Initialise the state that we want to live on the audio thread.
    let audio_host = Host::new();
    let fft_output = Arc::new(Mutex::new(1.0));
    let audio_model = Audio {
        sounds: Vec::new(),
        fft_output: Arc::clone(&fft_output),
    };

    // Create audio stream
    let audio_stream = audio_host
        .new_output_stream(audio_model)
        .sample_rate(48000)
        .render(audio)
        .build()
        .unwrap();

    // Generate a random point cloud
    let points = random_points();

    // Create the state
    let cloud_data = CloudData {
        sound_amplitude: 1.0,
        wind_strength: 0.2,
        noise_scale: 0.0,
        spring_constant: 0.002,
    };
    let state = State {
        cloud_file_path: "./data/union_station.e57".to_owned(),
        audio_file_path: "./data/audio.wav".to_owned(),
        movement_speed: 0.5,
        mouse_sensitivity: 0.003,
        cloud_data,
        // This will be accessed by the audio thread.
        fft_output,
    };

    // Create the camera
    let camera_config = CameraConfig::default().with_aspect_ratio(window_width, window_height);
    let camera = Camera::new(camera_config);

    // Initialise the shader pipeline
    let shader_pipeline = RefCell::new(GPUPipeline::new(&window, &points, camera, cloud_data));

    // Create the GUI
    let egui = Egui::from_window(&window);

    Model {
        window_id,
        egui,
        state,
        audio_stream,
        shader_pipeline,
        update_camera: RefCell::new(false),
        update_cloud_data: RefCell::new(false),
        camera_is_active,
    }
}

fn view(_app: &App, model: &Model, frame: Frame) {
    let mut pipeline = model.shader_pipeline.borrow_mut();

    // Check if the camera has been updated
    if *model.update_camera.borrow() {
        // Update the camera uniforms
        let device = frame.device_queue_pair().device();
        let encoder = &mut frame.command_encoder();
        pipeline.update_camera_transforms(device, encoder);
        *model.update_camera.borrow_mut() = false;
    }

    if *model.update_cloud_data.borrow() {
        let device = frame.device_queue_pair().device();
        let encoder = &mut frame.command_encoder();
        pipeline.update_cloud_data(device, encoder, model.state.cloud_data);
        *model.update_cloud_data.borrow_mut() = false;
    }

    pipeline.render(&frame);
    model.egui.draw_to_frame(&frame).unwrap();
}

fn update(app: &App, model: &mut Model, update: Update) {
    // Update GUI
    model.egui.set_elapsed_time(update.since_start);
    let window = app.window(model.window_id).unwrap();
    update_egui(model, window.device());

    // Get the audio strength
    let sound_amplitude = *model.state.fft_output.lock().unwrap();
    // Check if the sound amplitude has changed
    if model.state.cloud_data.sound_amplitude != sound_amplitude {
        model.state.cloud_data.sound_amplitude = sound_amplitude;
        *model.update_cloud_data.borrow_mut() = true;
    }

    // Update the camera position
    if model.camera_is_active {
        let mut pipeline = model.shader_pipeline.borrow_mut();
        let velocity = (update.since_last.secs() * model.state.movement_speed) as f32;

        // Update camera and update the model if the camera has moved
        if update_camera_position(pipeline.camera_mut(), velocity, &app.keys.down) {
            *model.update_camera.borrow_mut() = true;
        }
    }
}

fn audio(audio: &mut Audio, buffer: &mut Buffer) {
    let mut have_ended = vec![];
    let len_frames = buffer.len_frames();

    // Sum all of the sounds onto the buffer.
    audio.sounds.iter_mut().enumerate().for_each(|(i, sound)| {
        let mut frame_count = 0;
        let file_frames = sound.frames::<[f32; 2]>().filter_map(Result::ok);
        for (frame, file_frame) in buffer.frames_mut().zip(file_frames) {
            for (sample, file_sample) in frame.iter_mut().zip(&file_frame) {
                *sample += *file_sample;
            }
            frame_count += 1;
        }

        // If the sound yielded less samples than are in the buffer, it must have ended.
        if frame_count < len_frames {
            have_ended.push(i);
        }
    });

    // Remove all sounds that have ended.
    have_ended.into_iter().rev().for_each(|i| {
        audio.sounds.remove(i);
    });

    // Merge the audio channels and compute the FFT
    let samples: Vec<_> = buffer.frames().flatten().cloned().collect();
    let magnitude = compute_fft(&samples, buffer.sample_rate());

    // Update the audio strength value
    *audio.fft_output.lock().unwrap() = magnitude;
}

fn compute_fft(samples: &[f32], sample_rate: u32) -> f32 {
    // Apply hann window for smoothing; length must be a power of 2 for the FFT
    let hann_window = hann_window(samples);

    // Compute the FFT and get the spectrum
    let spectrum =
        samples_fft_to_spectrum(&hann_window, sample_rate, FrequencyLimit::Min(80.0), None).ok();

    // Compute the sum of the magnitudes
    match spectrum {
        Some(s) => s.data().iter().map(|f| f.1.val()).sum::<f32>().max(1.0),
        None => 1.0,
    }
}

fn update_egui(model: &mut Model, device: &wgpu::Device) {
    let ctx = model.egui.begin_frame();
    let state = &mut model.state;
    // Generate the settings window
    egui::Window::new("Settings")
        .default_width(0.0)
        .show(&ctx, |ui| {
            let prev_noise_scale = state.cloud_data.noise_scale;
            ui.label("noise_scale:");
            ui.add(egui::Slider::new(
                &mut state.cloud_data.noise_scale,
                0.0..=0.1,
            ));

            let prev_wind_strength = state.cloud_data.wind_strength;
            ui.label("wind_strength:");
            ui.add(egui::Slider::new(
                &mut state.cloud_data.wind_strength,
                0.0..=0.5,
            ));

            let prev_spring_constant = state.cloud_data.spring_constant;
            ui.label("spring_constant:");
            ui.add(egui::Slider::new(
                &mut state.cloud_data.spring_constant,
                0.0..=0.5,
            ));

            // Check if the cloud data has changed
            if prev_noise_scale != state.cloud_data.noise_scale
                || prev_wind_strength != state.cloud_data.wind_strength
                || prev_spring_constant != state.cloud_data.spring_constant
            {
                *model.update_cloud_data.borrow_mut() = true;
            }

            ui.label("movement_speed:");
            ui.add(egui::Slider::new(&mut state.movement_speed, 0.01..=1.0));

            ui.label("mouse_sensitivity:");
            ui.add(egui::Slider::new(
                &mut state.mouse_sensitivity,
                0.001..=0.01,
            ));

            ui.label("E57 path:");
            ui.text_edit_singleline(&mut state.cloud_file_path);

            let load_cloud = ui.button("Load file").clicked();
            if load_cloud {
                // Get the points from the E57 file if possible
                let points = if state.cloud_file_path.is_empty() {
                    random_points()
                } else {
                    match read_e57(&state.cloud_file_path) {
                        Ok(points) => points,
                        Err(_) => random_points(),
                    }
                };

                // Update the camera and points
                model
                    .shader_pipeline
                    .borrow_mut()
                    .new_point_cloud(device, &points);
                model
                    .shader_pipeline
                    .borrow_mut()
                    .camera_mut()
                    .fit_points(&points);
                *model.update_camera.borrow_mut() = true;
            }

            ui.label("Audio path:");
            ui.text_edit_singleline(&mut state.audio_file_path);

            if ui.button("Load file").clicked() {
                let audio_stream = &mut model.audio_stream;
                // Load the audio file if possible
                if let Ok(sound) = audrey::open(&state.audio_file_path) {
                    audio_stream
                        .send(move |audio| {
                            audio.sounds.clear();
                            audio.sounds.push(sound);
                        })
                        .ok();
                    audio_stream.play().unwrap();
                } else {
                    eprintln!("Failed to load audio file");
                };
            }
        });
}

fn update_camera_position(camera: &mut Camera, velocity: f32, keys: &keys::Down) -> bool {
    let mut moved = false;
    // Go forwards on W.
    if keys.contains(&Key::W) || keys.contains(&Key::Up) {
        camera.move_towards(Direction::Forward, velocity);
        moved = true;
    }
    // Go backwards on S.
    if keys.contains(&Key::S) || keys.contains(&Key::Down) {
        camera.move_towards(Direction::Backward, velocity);
        moved = true;
    }
    // Strafe left on A.
    if keys.contains(&Key::A) || keys.contains(&Key::Left) {
        camera.move_towards(Direction::Left, velocity);
        moved = true;
    }
    // Strafe right on D.
    if keys.contains(&Key::D) || keys.contains(&Key::Right) {
        camera.move_towards(Direction::Right, velocity);
        moved = true;
    }
    // Float down on Q.
    if keys.contains(&Key::Q) || keys.contains(&Key::Comma) {
        camera.move_towards(Direction::Down, velocity);
        moved = true;
    }
    // Float up on E.
    if keys.contains(&Key::E) || keys.contains(&Key::Period) {
        camera.move_towards(Direction::Up, velocity);
        moved = true;
    }

    moved
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    // Let egui handle things like keyboard and mouse input.
    model.egui.handle_raw_event(event);
}

fn key_pressed(app: &App, model: &mut Model, key: Key) {
    match key {
        Key::X | Key::Escape => app.quit(),
        Key::Space => {
            let window = app.main_window();
            if !model.camera_is_active {
                if window.set_cursor_grab(true).is_ok() {
                    model.camera_is_active = true;
                }
            } else if window.set_cursor_grab(false).is_ok() {
                model.camera_is_active = false;
            }
            window.set_cursor_visible(!model.camera_is_active);
        }
        Key::Z => app
            .main_window()
            .capture_frame(get_save_path(&app.exe_name().unwrap())),
        _other_key => {}
    }
}

fn event(_app: &App, model: &mut Model, event: Event) {
    if model.camera_is_active {
        if let Event::DeviceEvent(_device_id, winit::event::DeviceEvent::Motion { axis, value }) =
            event
        {
            let delta = -value as f32 * model.state.mouse_sensitivity;
            let mut pipeline = model.shader_pipeline.borrow_mut();
            let camera = pipeline.camera_mut();
            match axis {
                // Yaw left and right on mouse x axis movement.
                0 => camera.update_yaw(delta),
                // Pitch up and down on mouse y axis movement.
                _ => camera.update_pitch(delta),
            }
            *model.update_camera.borrow_mut() = true;
        }
    }
}
