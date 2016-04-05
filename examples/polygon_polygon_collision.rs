extern crate sfml;
extern crate box2d;
extern crate time;

// Latest version (1.11) of SFML for Rust requires a lot of traits to be included,
//   so just use the entirety of graphics
use sfml::graphics::*;
use sfml::system::Vector2f;
use sfml::window::{Key, VideoMode, event, window_style};

use box2d::world::World;
use box2d::body::BodyDef;
use box2d::body::BodyType;
use box2d::math::Vec2;

use time::Duration;
use time::get_time;

// Box2D tolerances are tuned to work in meters-kilogram-second (MKS) units.
// Using floating point numbers, shapes work best between 0.1 and 10 meters.
// It is HIGHLY recommended to have a meter-to-pixel scale to stay within these bounds.
// Here we will use 1 meter == 100 pixels.
const meters_to_pixels: f32 = 100.0;

fn main() {
    let step = 1.0 / 60.0;
    let mut current_time: f64 = 0.0;
    let mut accumulator: f64 = 0.0;

    // Create the window of the application
    let mut window = RenderWindow::new(VideoMode::new_init(800, 600, 32),
                                       "Bouncing Circle",
                                       window_style::CLOSE,
                                       &Default::default())
                         .expect("Cannot create a new Render Window.");
    // No tearing
    window.set_vertical_sync_enabled(true);

    let mut world = setup_box2d();
    let mut paused = true;

    while window.is_open() {
        // Handle events
        for event in window.events() {
            match event {
                event::Closed => window.close(),
                event::KeyPressed { code, .. } => {
                    match code {
                        Key::Escape => return,
                        Key::Space => paused = !paused,
                        _ => {}
                    }
                },
                _             => {/* do nothing */}
            }
        }

        let time_data = get_time();
        let new_time = (Duration::seconds(time_data.sec as i64) + Duration::nanoseconds(time_data.nsec as i64)).num_milliseconds() as f64 / 1000.0;
        // Don't count frame_time when paused
        let frame_time = if !paused { (new_time - current_time).min(0.2) } else { 0.0 };
        current_time = new_time;

        accumulator = accumulator + frame_time;
        while accumulator >= step && !paused {
            world.step(step as f32);
            accumulator -= step;
        }

        // Clear the window
        window.clear(&Color::new_rgb(0, 200, 200));
        for i in 0..world.bodies.len() {
            let ref shape = world.bodies[i].shape;
            match *shape {
                box2d::shape::shape::Shape::CircleShape{center, radius} => {
                    let mut circle = CircleShape::new().expect("Error, cannot create ball.");
                    // Units in Box2D should be converted from Meters to Pixels
                    let position = (world.bodies[i].position + center).multiply(meters_to_pixels);
                    let radius = radius * meters_to_pixels;
                    circle.set_radius(radius-1.0);
                    circle.set_outline_thickness(1.0);
                    circle.set_outline_color(&Color::new_rgb(255, 0, 0));
                    circle.set_fill_color(&Color::transparent());
                    circle.set_position(&Vector2f::new(position.x, position.y));
                    circle.set_origin(&Vector2f::new(radius, radius));
                    window.draw(&circle);
                },

                box2d::shape::shape::Shape::LineShape{point1, point2} => {
                    // Units in Box2D should be converted from Meters to Pixels
                    let point1_global = (world.bodies[i].position + point1).multiply(meters_to_pixels);
                    let point2_global = (world.bodies[i].position + point2).multiply(meters_to_pixels);
                    
                    // Latest SFML uses new type, VertexArray, to draw primitive types
                    let mut points = VertexArray::new().unwrap();
                    points.set_primitive_type(Lines);
                    points.append(&Vertex::new_with_pos_color(&Vector2f {
                                                                    x: point1_global.x,
                                                                    y: point1_global.y
                                                                },
                                                                &Color::blue()));
                    points.append(&Vertex::new_with_pos_color(&Vector2f {
                                                                    x: point2_global.x,
                                                                    y: point2_global.y
                                                                }, &Color::blue()));
                    window.draw(&points);
                },
                box2d::shape::shape::Shape::ChainLineShape{ref points} => {
                    // Latest SFML uses new type, VertexArray, to draw primitive types
                    let mut global_points = VertexArray::new().unwrap();
                    global_points.set_primitive_type(LinesStrip);
                    for p in points.iter() {
                        // Units in Box2D should be converted from Meters to Pixels
                        let mut global_point = (world.bodies[i].position + *p).multiply(meters_to_pixels);
                        global_points.append(&Vertex::new_with_pos_color(&Vector2f {
                                                                                x: global_point.x,
                                                                                y: global_point.y
                                                                            },
                                                                            &Color::blue()));
                    }
                    window.draw(&global_points);
                },
                box2d::shape::shape::Shape::PolygonShape{ref points} => {
                    // Latest SFML uses new type, VertexArray, to draw primitive types
                    let mut global_points = VertexArray::new().unwrap();
                    global_points.set_primitive_type(LinesStrip);
                    for p in points.iter() {
                        // Units in Box2D should be converted from Meters to Pixels
                        let global_point = (world.bodies[i].position + *p).multiply(meters_to_pixels);
                        global_points.append(&Vertex::new_with_pos_color(&Vector2f {
                                                                            x: global_point.x,
                                                                            y: global_point.y
                                                                        },
                                                                        &Color::red()));
                    }
                    // Close off polygon by adding first point to end
                    let global_point = (world.bodies[i].position + points[0]).multiply(meters_to_pixels);
                    global_points.append(&Vertex::new_with_pos_color(&Vector2f {
                                                                            x: global_point.x,
                                                                            y: global_point.y
                                                                        },
                                                                        &Color::red()));
                    window.draw(&global_points);
                }
            }
        }
        window.display();
    }
}

fn setup_box2d() -> World {
    // Safe to use the box2d shapes here, be aware, though, SFML has similar shape names.
    use box2d::shape::shape::Shape::*;

    // Earth gravity, 9.8m/s
    let mut world = World::new(Vec2::new(0.0, 9.8));

    // Units in Box2D are 1.0 == 1 meter.
    let polygon_shape = PolygonShape {
                            points: vec! [
                                Vec2::new(-0.75, -0.75),
                                Vec2::new(-0.75, 0.75),
                                Vec2::new(0.75, 0.75),
                                Vec2::new(0.75, -0.75)
                        ]};
    let polygon_body_def = BodyDef {
                            shape: polygon_shape,
                            body_type: BodyType::StaticBody,
                            position: Vec2::new(4.0, 4.0),
                            velocity: Vec2::new(0.0, 0.0),
                            restitution: 1.0,
                            mass: 0.0,
                            gravity_scale: 1.0
                        };
    world.add_body(polygon_body_def);

    let polygon_shape2 = PolygonShape {
                            points: vec! [
                                Vec2::new(-0.75, -0.75),
                                Vec2::new(-0.75, 0.75),
                                Vec2::new(0.75, 0.75),
                                Vec2::new(0.75, -0.75)
                        ]};
    let polygon_body_def2 = BodyDef {
                            shape: polygon_shape2,
                            body_type: BodyType::DynamicBody,
                            position: Vec2::new(3.5, 1.0),
                            velocity: Vec2::new(0.0, 0.0),
                            restitution: 1.0,
                            mass: 1.0,
                            gravity_scale: 1.0
                        };
    world.add_body(polygon_body_def2);

    return world;
}
