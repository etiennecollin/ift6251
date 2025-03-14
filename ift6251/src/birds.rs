use ift6251::get_save_path;
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

#[derive(Debug, Clone, Copy)]
struct Agent {
    position: Vec2,
    velocity: Vec2,
}

impl Agent {
    const SIZE: (f32, f32) = (15.0, 15.0);
    const COLOR: (f32, f32, f32, f32) = (1.0, 1.0, 1.0, 1.0);
    const NUM_AGENTS: usize = 200;

    const SPEED: f32 = 1.5;
    const DETECTION_RADIUS: f32 = 60.0;
    const MIN_DISTANCE: f32 = 30.0;
    const MIN_DISTANCE_INVERSE: f32 = 1.0 / Self::MIN_DISTANCE;

    const MIN_DISTANCE_FACTOR: f32 = 0.3;
    const AVERAGE_VELOCITY_FACTOR: f32 = 0.01;
    const AVERAGE_POSITION_FACTOR: f32 = 1e-4;

    fn new(win_rect: Rect) -> Self {
        // Random position and velocity
        let position = vec2(
            random_range(win_rect.left(), win_rect.right()),
            random_range(win_rect.top(), win_rect.bottom()),
        );
        let velocity =
            Vec2::new(random_range(-1.0, 1.0), random_range(-1.0, 1.0)).normalize() * Self::SPEED;

        // Return new agent
        Agent { position, velocity }
    }
    fn step(&mut self, win_rect: &Rect) {
        self.position += self.velocity;

        // Wrap around screen width
        if self.position.x < win_rect.left() {
            self.position.x = win_rect.right();
        } else if self.position.x > win_rect.right() {
            self.position.x = win_rect.left();
        }

        // Wrap around screen height
        if self.position.y > win_rect.top() {
            self.position.y = win_rect.bottom();
        } else if self.position.y < win_rect.bottom() {
            self.position.y = win_rect.top();
        }
    }

    fn update(&mut self, win_rect: Rect, agents: &Vec<Agent>) {
        // Move agent
        self.step(&win_rect);

        // Calculate average position, velocity and separation of neighbors
        // and adjust the agent's velocity accordingly
        let mut average_position = Vec2::default();
        let mut average_velocity = Vec2::default();
        let mut num_neighbors = 0;

        // Iterate over all agents
        for other in agents {
            let distance = self.position.distance(other.position);

            // Check if other agent is in the detection range and not the agent itself
            if distance < Self::DETECTION_RADIUS && distance > 0.0 {
                average_velocity += other.velocity;
                average_position += other.position;

                // Make sure to keep a minimum distance to other agents
                if distance < Self::MIN_DISTANCE {
                    // Move agent away from other agent
                    // The closer the agent, the stronger the force
                    average_velocity += average_velocity.perp()
                        * Self::MIN_DISTANCE_FACTOR
                        * distance
                        * Self::MIN_DISTANCE_INVERSE;
                }
                num_neighbors += 1;
            }
        }

        // Calculate average position, velocity and separation
        if num_neighbors > 0 {
            average_position /= num_neighbors as f32;
            average_velocity /= num_neighbors as f32;
        }

        // Agent should move towards the same direction as its neighbors
        self.velocity = self
            .velocity
            .lerp(average_velocity, Self::AVERAGE_VELOCITY_FACTOR);

        // Agent should move towards the average position of its neighbors to stay with them
        self.velocity += (average_position - self.position) * Self::AVERAGE_POSITION_FACTOR;

        // Normalize velocity and set speed
        self.velocity = self.velocity.normalize() * Self::SPEED;
    }

    fn display(&self, draw: &Draw, color: Rgba) {
        draw.tri()
            .xy(self.position)
            .rotate(self.velocity.angle())
            .wh(Self::SIZE.into())
            .color(color);
    }
}

struct Model {
    agents: Vec<Agent>,
}

fn model(app: &App) -> Model {
    app.new_window()
        .title("Birds")
        .fullscreen()
        .view(view)
        .key_pressed(key_pressed)
        .build()
        .unwrap();

    let agents = (0..Agent::NUM_AGENTS)
        .map(|_| Agent::new(app.window_rect()))
        .collect();

    Model { agents }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let previous_agents = model.agents.clone();
    model
        .agents
        .iter_mut()
        .for_each(|agent| agent.update(app.window_rect(), &previous_agents));
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();

    // Clear the background to black
    draw.background().color(BLACK);

    // Draw agents
    model.agents.iter().for_each(|agent| {
        agent.display(&draw, Agent::COLOR.into());
    });

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
