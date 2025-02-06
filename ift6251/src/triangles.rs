// Inspired by: https://www.local-guru.net/blog/2020/12/19/nannou-experiment
//
// Cool values:
// Noise scale x 0.03218235128341096
// Noise scale y 0.06194991627131155
// Noise scale w 0.07323387803617847
// Noise scale h 0.06021031392057739
// Noise scale time xy 0.0007596394736761769
// Noise scale time wh 0.01868244391839513
// Perlin seed 0
// Stroke color HSLA=(RgbHue(273.71014), 0.54207826, 0.23118138, 0.1)
// Fill color HSLA=(RgbHue(332.50726), 0.7435478, 0.27593488, 0.01)

use nannou::{
    color::Hue,
    noise::{NoiseFn, Perlin, Seedable},
    prelude::*,
};
use nannou_egui::{egui, Egui, FrameCtx};

fn main() {
    nannou::app(model).update(update).run()
}

struct State {
    position: Vec2,
    size: Vec2,
    rotation: f32,
    roll: f32,
}

struct Settings {
    noise_scale_x: f64,
    noise_scale_y: f64,
    noise_scale_w: f64,
    noise_scale_h: f64,
    noise_scale_time_xy: f64,
    noise_scale_time_wh: f64,
    rotation_increment: f32,
    stroke_color: Hsla,
    fill_color: Hsla,
    noise: Perlin,
}

struct Model {
    egui: Egui,
    settings: Settings,
    state: State,
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .fullscreen()
        .view(view)
        .key_pressed(key_pressed)
        .raw_event(raw_window_event)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();
    let egui = Egui::from_window(&window);

    let settings = Settings {
        noise_scale_x: 0.1,
        noise_scale_y: 0.1,
        noise_scale_w: 0.042,
        noise_scale_h: 0.042,
        noise_scale_time_xy: 0.004,
        noise_scale_time_wh: 0.01,
        rotation_increment: 0.001,
        stroke_color: hsla(0.0, 1.0, 0.5, 0.1),
        fill_color: hsla(0.0, 1.0, 0.01, 0.1),
        noise: Perlin::new(),
    };

    let state = State {
        position: vec2(0.0, 0.0),
        size: vec2(0.0, 0.0),
        rotation: 0.0,
        roll: 0.0,
    };

    Model {
        egui,
        settings,
        state,
    }
}

fn update_egui(ctx: FrameCtx, settings: &mut Settings) {
    // Generate the settings window
    egui::Window::new("Settings").show(&ctx, |ui| {
        ui.label("Noise scale x:");
        ui.add(egui::Slider::new(&mut settings.noise_scale_x, 0.00..=0.1));

        ui.label("Noise scale y:");
        ui.add(egui::Slider::new(&mut settings.noise_scale_y, 0.00..=0.1));

        ui.label("Noise scale w:");
        ui.add(egui::Slider::new(&mut settings.noise_scale_w, 0.00..=0.1));

        ui.label("Noise scale h:");
        ui.add(egui::Slider::new(&mut settings.noise_scale_h, 0.00..=0.1));

        ui.label("Noise scale time xy:");
        ui.add(egui::Slider::new(
            &mut settings.noise_scale_time_xy,
            0.000..=0.05,
        ));

        ui.label("Noise scale time wh:");
        ui.add(egui::Slider::new(
            &mut settings.noise_scale_time_wh,
            0.000..=0.05,
        ));

        ui.label("Rotation increment:");
        ui.add(egui::Slider::new(
            &mut settings.rotation_increment,
            0.00..=1.00,
        ));

        let rnd_color = ui.button("Random color").clicked();
        if rnd_color {
            settings.stroke_color = hsla(random(), random(), random(), 0.1);
            settings.fill_color = hsla(random(), random(), random(), 0.01);
        }

        let rnd_noise = ui.button("Random noise values").clicked();
        if rnd_noise {
            settings.noise_scale_x = random_range(0.0, 0.1);
            settings.noise_scale_y = random_range(0.0, 0.1);
            settings.noise_scale_w = random_range(0.0, 0.1);
            settings.noise_scale_h = random_range(0.0, 0.1);
            settings.noise_scale_time_xy = random_range(0.0, 0.05);
            settings.noise_scale_time_wh = random_range(0.0, 0.05);
        }

        let rnd_perlin_seed = ui.button("Random Perlin seed").clicked();
        if rnd_perlin_seed {
            settings.noise.set_seed(random());
        }

        let save_settings = ui.button("Save settings").clicked();
        if save_settings {
            println!("Noise scale x {}", settings.noise_scale_x);
            println!("Noise scale y {}", settings.noise_scale_y);
            println!("Noise scale w {}", settings.noise_scale_w);
            println!("Noise scale h {}", settings.noise_scale_h);
            println!("Noise scale time xy {}", settings.noise_scale_time_xy);
            println!("Noise scale time wh {}", settings.noise_scale_time_wh);
            println!("Perlin seed {}", settings.noise.seed());
            println!(
                "Stroke color HSLA=({:?}, {:?}, {:?}, {:?})",
                settings.stroke_color.hue,
                settings.stroke_color.saturation,
                settings.stroke_color.lightness,
                settings.stroke_color.alpha
            );
            println!(
                "Fill color HSLA=({:?}, {:?}, {:?}, {:?})",
                settings.fill_color.hue,
                settings.fill_color.saturation,
                settings.fill_color.lightness,
                settings.fill_color.alpha
            );
        }
    });
}

fn update(app: &App, model: &mut Model, update: Update) {
    let egui = &mut model.egui;
    let settings = &mut model.settings;
    let state = &mut model.state;

    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();
    update_egui(ctx, settings);

    // Compute a subsection of the window size
    let window_width = (app.window_rect().w() / 4.0) as f64;
    let window_height = (app.window_rect().top() / 4.0) as f64;

    // Scale the elapsed frames for different noise scales
    let t_wh = app.elapsed_frames() as f64 * settings.noise_scale_time_wh;
    let t_xy = app.elapsed_frames() as f64 * settings.noise_scale_time_xy;

    // Noisy values for width and height of the triangle
    let w = (t_wh * settings.noise_scale_w).cos() * window_width + 100.0;
    let h = (t_wh * settings.noise_scale_h).sin() * window_height + 100.0;

    // Noisy values for x and y position of the triangle
    let x = settings.noise.get([-(t_xy * settings.noise_scale_x), t_xy]) * window_width;
    let y = settings.noise.get([t_xy, (t_xy * settings.noise_scale_y)]) * window_height;

    // Increment the rotation and roll of the triangle
    let rotation = (state.rotation + settings.rotation_increment) % (2.0 * PI);
    let roll = (state.rotation + settings.rotation_increment) % (2.0 * PI);

    // Evolution of the colors
    settings.stroke_color = settings.stroke_color.shift_hue(0.08);
    settings.fill_color = settings.fill_color.shift_hue(0.05);

    // Update the state
    state.position = vec2(x as f32, y as f32);
    state.size = vec2(w as f32, h as f32);
    state.rotation = rotation;
    state.roll = roll;
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    // Let egui handle things like keyboard and mouse input.
    model.egui.handle_raw_event(event);
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Initial frame should be cleared black
    if frame.nth() == 0 {
        frame.clear(BLACK);
    }

    let draw = app.draw();
    let window = app.window_rect();
    let state = &model.state;
    let settings = &model.settings;

    // Slowly fade the previous frame
    draw.rect()
        .x_y(0.0, 0.0)
        .w_h(window.w(), window.h())
        .color(hsla(0.0, 0.0, 0.0, 0.005));

    // Draw the triangle
    draw.tri()
        .xy(state.position)
        .wh(state.size)
        .rotate(state.rotation)
        .roll(state.roll)
        .color(settings.fill_color)
        .stroke(settings.stroke_color)
        .stroke_weight(1.0);

    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}

fn key_pressed(app: &App, _model: &mut Model, key: Key) {
    match key {
        Key::Q => app.quit(),
        Key::S => {
            app.main_window()
                .capture_frame(app.exe_name().unwrap() + ".png");
        }
        _other_key => {}
    }
}
