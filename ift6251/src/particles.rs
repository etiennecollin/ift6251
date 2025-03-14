// Inspired by:
// The Nature of Code - Daniel Shiffman
// http://natureofcode.com

use ift6251::get_save_path;
use nannou::{
    noise::{NoiseFn, Perlin},
    prelude::*,
};

const INITIAL_PARTICLE_COUNT: u64 = 1000;

fn main() {
    nannou::app(model).update(update).run();
}

// A simple particle type
#[derive(Clone)]
struct Particle {
    position: Point2,
    velocity: Vec2,
    acceleration: Vec2,
    life_span: f32,
    radius: f32,
    mass: f32,
    id: u64,
}

impl Particle {
    const LIFE_SPAN_MAX: f32 = 512.0;
    const LIFE_SPAN_DELTA: f32 = 0.5;
    const MASS_MIN: f32 = 1.0;
    const MASS_MAX: f32 = 10.0;
    const RESTITUTION_COEFFICIENT: f32 = 0.8;
    const GRAVITATIONAL_CONSTANT: f32 = 0.025;
    const RADIUS: f32 = 2.0;

    fn new(position: Point2, id: u64) -> Self {
        let mass = random_range(Self::MASS_MIN, Self::MASS_MAX);
        let radius = Self::RADIUS * mass / (4.0 * Self::MASS_MIN);
        // let radius = Self::RADIUS;
        Particle {
            acceleration: Vec2::ZERO,
            velocity: vec2(random_range(-1.0, 1.0), random_range(-1.0, 1.0)),
            position,
            life_span: Self::LIFE_SPAN_MAX,
            radius,
            mass,
            id,
        }
    }

    fn interacts(&mut self, particles: &[Particle]) {
        particles.iter().for_each(|particle| {
            if particle.id != self.id {
                // Compute the distance between the particles
                let direction = self.position - particle.position;
                let distance = direction.length();
                let distance_inverse = 1.0 / distance.powi(2).max(f32::EPSILON);

                // Elastic collisions
                if distance <= self.radius + particle.radius {
                    // If they collide, calculate the new velocity after the elastic collision
                    let m1 = self.mass;
                    let m2 = particle.mass;
                    let v1 = self.velocity;
                    let v2 = particle.velocity;

                    // Calculate the relative velocity
                    let relative_velocity = v1 - v2;
                    let dot_product = relative_velocity.dot(direction);

                    // Calculate the new velocity for particle 1 after the elastic collision
                    let force = -((2.0 * m2 / (m1 + m2)) * dot_product * distance_inverse)
                        * direction
                        * Self::RESTITUTION_COEFFICIENT;

                    self.apply_force(force);
                    return; // Skip further processing if particles are overlapping
                }

                // Gravitational interaction
                let force = direction.normalize()
                    * (Self::GRAVITATIONAL_CONSTANT
                        * (self.mass * particle.mass)
                        * distance_inverse);
                self.apply_force(force);
            }
        });
    }

    fn check_bounds(&mut self, bounds: &Rect) {
        // Bounce off the bounds of the window
        if self.position.x > bounds.right() {
            self.position.x = bounds.right();
            self.velocity.x *= -1.0;
        } else if self.position.x < bounds.left() {
            self.position.x = bounds.left();
            self.velocity.x *= -1.0;
        }

        if self.position.y > bounds.top() {
            self.position.y = bounds.top();
            self.velocity.y *= -1.0;
        } else if self.position.y < bounds.bottom() {
            self.position.y = bounds.bottom();
            self.velocity.y *= -1.0;
        }
    }

    fn apply_force(&mut self, f: Vec2) {
        self.acceleration += f;
    }

    // Method to update position
    fn update(&mut self) {
        self.velocity += self.acceleration;
        self.position -= self.velocity;
        self.acceleration = Vec2::ZERO;
        self.life_span -= Self::LIFE_SPAN_DELTA;
    }

    // Method to display
    fn display(&self, draw: &Draw) {
        let mass_color = self.mass / Self::MASS_MAX;
        draw.ellipse().xy(self.position).radius(self.radius).rgba(
            mass_color,
            0.0,
            0.0,
            self.life_span / 255.0,
        );
    }

    // Is the particle still useful?
    fn is_dead(&self) -> bool {
        self.life_span <= 0.0
    }
}

struct ParticleSystem {
    bounds: Rect,
    particles: Vec<Particle>,
    noise: Perlin,
}

impl ParticleSystem {
    const NOISE_SCALE: f64 = 0.0008;
    const NOISE_FORCE_MULTIPLIER: f32 = 0.1;

    fn new(bounds: Rect) -> Self {
        ParticleSystem {
            bounds,
            particles: Vec::new(),
            noise: Perlin::new(),
        }
    }

    fn add_particle(&mut self, origin: Point2, id: u64) {
        self.particles.push(Particle::new(origin, id));
    }

    fn update(&mut self) {
        let particles = self.particles.clone();

        // Update status of all particles and remove dead ones.
        // Also handle interatctions between particles.
        // We iterate in reverse order to be able to remove particles
        // from the vector while iterating.
        for i in (0..self.particles.len()).rev() {
            let particle = &mut self.particles[i];

            // Check bounds
            particle.check_bounds(&self.bounds);

            // Apply force field
            // let x = particle.position.x as f64 * Self::NOISE_SCALE;
            // let y = particle.position.y as f64 * Self::NOISE_SCALE;
            // let vx = particle.velocity.x as f64 * Self::NOISE_SCALE;
            // let vy = particle.velocity.y as f64 * Self::NOISE_SCALE;
            // let force_x = self.noise.get([x, y]) as f32;
            // let force_y = self.noise.get([vx, vy]) as f32;
            // let force = vec2(force_x, force_y) * Self::NOISE_FORCE_MULTIPLIER;
            // particle.apply_force(force);

            // Interactions between particles
            particle.interacts(&particles);

            // Update particle
            particle.update();

            // Remove particle if dead
            if particle.is_dead() {
                self.particles.remove(i);
            }
        }
    }

    fn draw(&self, draw: &Draw) {
        self.particles
            .iter()
            .for_each(|particle| particle.display(draw));
    }
}

struct Model {
    ps: ParticleSystem,
}

fn model(app: &App) -> Model {
    app.new_window()
        .title("Scratch")
        .fullscreen()
        .view(view)
        .key_pressed(key_pressed)
        .build()
        .unwrap();

    let mut ps = ParticleSystem::new(app.window_rect());

    let bounds = app.window_rect();
    (0..INITIAL_PARTICLE_COUNT).for_each(|id| {
        let origin = pt2(
            random_range(bounds.left(), bounds.right()),
            random_range(bounds.bottom(), bounds.top()),
        );
        ps.add_particle(origin, id);
    });

    Model { ps }
}

fn update(app: &App, m: &mut Model, _update: Update) {
    // Add a new particle
    let bounds = app.window_rect();
    let origin = pt2(
        random_range(bounds.left(), bounds.right()),
        random_range(bounds.bottom(), bounds.top()),
    );
    m.ps.add_particle(origin, app.elapsed_frames());

    // Update the particle system
    m.ps.update();
}

fn view(app: &App, m: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    m.ps.draw(&draw);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}

fn key_pressed(app: &App, _model: &mut Model, key: Key) {
    match key {
        Key::Q => app.quit(),
        Key::S => {
            app.main_window()
                .capture_frame(get_save_path(&app.exe_name().unwrap()));
        }
        _other_key => {}
    }
}
