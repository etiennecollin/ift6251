use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

use nannou::{
    noise::{NoiseFn, Perlin},
    prelude::*,
    state::keys,
    winit,
};
use nannou_audio::{Buffer, Host, Stream};
use nannou_egui::{
    Egui,
    egui::{self},
};
use point_cloud_renderer::{
    camera::{Camera, Direction},
    loader::{generate_random_point_cloud, read_e57},
    pipeline::GPUPipeline,
    point::Point,
};
use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};
use spectrum_analyzer::{FrequencyLimit, samples_fft_to_spectrum, windows::hann_window};

fn main() {
    nannou::app(model).event(event).update(update).run();
}

struct State {
    cloud_file_path: String,
    audio_file_path: String,
    initial_points: Vec<Point>,
    points: Vec<Point>,
    movement_speed: f64,
    mouse_sensitivity: f32,
    noise: Perlin,
    noise_scale: f32,
    wind_strength: f32,
    spring_constant: f32,
    // This will be accessed by the audio thread.
    _volume: Arc<Mutex<f32>>,
    fft_output: Arc<Mutex<f32>>,
}

struct Audio {
    sounds: Vec<audrey::read::BufFileReader>,
    volume: Arc<Mutex<f32>>,
    fft_output: Arc<Mutex<f32>>,
}

struct Model {
    window_id: WindowId,
    egui: Egui,
    state: State,
    audio_stream: Stream<Audio>,
    shader_pipeline: RefCell<GPUPipeline>,
    update_points: bool,
    update_camera: RefCell<bool>,
    camera_is_active: bool,
}

fn random_points() -> Vec<Point> {
    let range_x = (-100.0, 100.0);
    let range_y = (-100.0, 100.0);
    let range_z = (-100.0, 100.0);
    generate_random_point_cloud(500000, range_x, range_y, range_z)
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .fullscreen()
        .view(view)
        .raw_event(raw_window_event)
        .key_pressed(key_pressed)
        .build()
        .unwrap();
    let window = app.window(window_id).unwrap();

    // Get control of the cursor
    let camera_is_active = true;
    if let Err(e) = window.set_cursor_grab(true) {
        eprintln!("warning: cursor grabbing not supported: {e}");
    }
    window.set_cursor_visible(false);

    // Generate a random point cloud
    let points = random_points();

    // Create the camera
    let eye = Point3::new(0.0, 0.0, 1.0);
    let camera = Camera::new(eye);

    // Initialise the shader pipeline
    let shader_pipeline = RefCell::new(GPUPipeline::new(&window, &points, camera));

    // Initialise the state that we want to live on the audio thread.
    let audio_host = Host::new();
    let volume = Arc::new(Mutex::new(0.0));
    let fft_output = Arc::new(Mutex::new(1.0));
    let audio_model = Audio {
        sounds: vec![],
        fft_output: Arc::clone(&fft_output),
        volume: Arc::clone(&volume),
    };

    // Create audio stream
    let audio_stream = audio_host
        .new_output_stream(audio_model)
        .render(audio)
        .build()
        .unwrap();

    // Create noise
    let noise = Perlin::new();

    // Create the state
    let state = State {
        cloud_file_path: "./data/union_station.e57".to_owned(),
        audio_file_path: "./data/audio.wav".to_owned(),
        initial_points: points.clone(),
        points,
        movement_speed: 0.5,
        mouse_sensitivity: 0.003,
        noise,
        noise_scale: 0.0,
        wind_strength: 0.2,
        spring_constant: 0.002,
        // This will be accessed by the audio thread.
        _volume: volume,
        fft_output,
    };

    // Create the GUI
    let egui = Egui::from_window(&window);

    Model {
        window_id,
        egui,
        state,
        audio_stream,
        shader_pipeline,
        update_points: false,
        update_camera: RefCell::new(false),
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
        pipeline.update_uniforms(device, encoder);
        *model.update_camera.borrow_mut() = false;
    }

    pipeline.render(&frame);
    model.egui.draw_to_frame(&frame).unwrap();
}

fn audio(audio: &mut Audio, buffer: &mut Buffer) {
    let mut have_ended = vec![];
    let len_frames = buffer.len_frames();
    let mut rms_volume = 0.0;

    // Sum all of the sounds onto the buffer.
    for (i, sound) in audio.sounds.iter_mut().enumerate() {
        let mut frame_count = 0;
        let file_frames = sound.frames::<[f32; 2]>().filter_map(Result::ok);
        for (frame, file_frame) in buffer.frames_mut().zip(file_frames) {
            let mut frame_rms = 0.0; // Compute the root mean square of the frame
            for (sample, file_sample) in frame.iter_mut().zip(&file_frame) {
                *sample += *file_sample;
                frame_rms += file_sample.powi(2);
            }
            rms_volume += (frame_rms / 2.0).sqrt();
            frame_count += 1;
        }

        // If the sound yielded less samples than are in the buffer, it must have ended.
        if frame_count < len_frames {
            have_ended.push(i);
        }
    }

    // Remove all sounds that have ended.
    for i in have_ended.into_iter().rev() {
        audio.sounds.remove(i);
    }

    // Normalize the volume
    let volume = rms_volume / len_frames as f32 * 100.0;
    // Update the volume value
    *audio.volume.lock().unwrap() = volume;

    // Merge the audio channels
    let fft_input: Vec<f32> = buffer.frames().flatten().cloned().collect();

    // Apply hann window for smoothing; length must be a power of 2 for the FFT
    let hann_window = hann_window(&fft_input);

    // Compute the FFT and get the spectrum
    let spectrum = samples_fft_to_spectrum(
        &hann_window,
        buffer.sample_rate(),
        FrequencyLimit::All,
        None,
    )
    .ok();

    // Compute the sum of the magnitudes
    let magnitude = match spectrum {
        Some(s) => s.data().par_iter().map(|f| f.1.val()).sum::<f32>().max(1.0),
        None => 1.0,
    };

    // Update the audio strength value
    *audio.fft_output.lock().unwrap() = magnitude;
}

fn update(app: &App, model: &mut Model, update: Update) {
    let time = update.since_start.secs();

    // Update GUI
    model.egui.set_elapsed_time(update.since_start);
    update_egui(model);

    // Get the audio strength
    let audio_strength = *model.state.fft_output.lock().unwrap();
    flow(&mut model.state, time, audio_strength);

    // Update the points on the GPU
    let window = app.window(model.window_id).unwrap();
    let mut pipeline = model.shader_pipeline.borrow_mut();
    pipeline.update_points(window.device(), &model.state.points);
    model.update_points = false;

    // Update the camera position
    if model.camera_is_active {
        let velocity = (update.since_last.secs() * model.state.movement_speed) as f32;
        update_camera_position(pipeline.camera_mut(), velocity, &app.keys.down);
        *model.update_camera.borrow_mut() = true;
    }
}

fn flow(state: &mut State, time: f64, audio_strength: f32) {
    let noise = &mut state.noise;
    let points = &mut state.points;
    let initial_points = &state.initial_points; // Use the initial positions
    let scale = state.noise_scale as f64;
    let scaled_time = time * scale;
    let wind_strength = state.wind_strength * audio_strength;
    let spring_constant = state.spring_constant;

    if wind_strength == 0.0 && spring_constant == 0.0 {
        return;
    }

    points.par_iter_mut().enumerate().for_each(|(i, point)| {
        let initial_position = initial_points[i].position; // Get the original position of the point
        let x = point.position[0] as f64 * scale;
        let y = point.position[1] as f64 * scale;
        let z = point.position[2] as f64 * scale;

        // Simulate wind-like vector field using noise or another function
        let wind = noise.get([x, y, z, scaled_time]) as f32 * wind_strength;

        // Apply wind force to the point's position
        point.position[0] += wind;
        point.position[1] += wind;
        point.position[2] += wind;

        // Calculate the distance from the original position
        let displacement = [
            point.position[0] - initial_position[0],
            point.position[1] - initial_position[1],
            point.position[2] - initial_position[2],
        ];

        // Apply the spring-like restorative force
        point.position[0] -= spring_constant * displacement[0];
        point.position[1] -= spring_constant * displacement[1];
        point.position[2] -= spring_constant * displacement[2];
    });
}

fn update_camera_position(camera: &mut Camera, velocity: f32, keys: &keys::Down) {
    // Go forwards on W.
    if keys.contains(&Key::W) {
        camera.move_towards(Direction::Forward, velocity);
    }
    // Go backwards on S.
    if keys.contains(&Key::R) {
        camera.move_towards(Direction::Backward, velocity);
    }
    // Strafe left on A.
    if keys.contains(&Key::A) {
        camera.move_towards(Direction::Left, velocity);
    }
    // Strafe right on D.
    if keys.contains(&Key::S) {
        camera.move_towards(Direction::Right, velocity);
    }
    // Float down on Q.
    if keys.contains(&Key::Q) {
        camera.move_towards(Direction::Down, velocity);
    }
    // Float up on E.
    if keys.contains(&Key::F) {
        camera.move_towards(Direction::Up, velocity);
    }
}

fn update_egui(model: &mut Model) {
    let ctx = model.egui.begin_frame();
    let state = &mut model.state;
    let audio_stream = &mut model.audio_stream;

    // Generate the settings window
    egui::Window::new("Settings")
        .default_width(0.0)
        .show(&ctx, |ui| {
            ui.label("noise_scale:");
            ui.add(egui::Slider::new(&mut state.noise_scale, 0.0..=0.1));

            ui.label("wind_strength:");
            ui.add(egui::Slider::new(&mut state.wind_strength, 0.0..=0.5));

            ui.label("spring_constant:");
            ui.add(egui::Slider::new(&mut state.spring_constant, 0.0..=0.5));

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
                state.initial_points = points.clone();
                state.points = points;
                model.update_points = true;
            }

            ui.label("Audio path:");
            ui.text_edit_singleline(&mut state.audio_file_path);

            let load_audio = ui.button("Load file").clicked();
            if load_audio {
                // Load the audio file if possible
                if let Ok(sound) = audrey::open(state.audio_file_path.clone()) {
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

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    // Let egui handle things like keyboard and mouse input.
    model.egui.handle_raw_event(event);
}

fn key_pressed(app: &App, model: &mut Model, key: Key) {
    match key {
        Key::X => app.quit(),
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
        }
    }
}
