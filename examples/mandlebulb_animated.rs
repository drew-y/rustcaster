extern crate rustcaster;

use rustcaster::render::render_animation;
use rustcaster::scene::{AnimatedScene, RayMarchOpts, Scene};
use rustcaster::vec3::*;
use std::sync::Arc;

//////////////////////
// Raymarch Config
//////////////////////
const MAX_STEPS: i32 = 500;
const MIN_HIT_DIST: f64 = 0.0001;
const MAX_TRACE_DIST: f64 = 2000.0;
const DARKNESS: f64 = 35.0;

//////////////////////
// MandleBulb Config
//////////////////////
const MANDLEBULB_ITERATIONS: i32 = 400;
const MANDLEBULB_SCALE: f64 = 1.0;

fn saturate(color: Vec3) -> Vec3 {
    vec3(
        color.x.clamp(0.0, 1.0),
        color.y.clamp(0.0, 1.0),
        color.z.clamp(0.0, 1.0),
    )
}

/// Returns (distance, iterations)
fn mandlebulb_distance(pos: Vec3, time: f64) -> (f64, f64) {
    let mut z = pos;
    let mut dr = 1.0f64;
    let mut r = 0.0f64;
    let power = 1.0f64 + ((time / 25.0).sin() * 10.0).abs();

    let mut iterations = 0;
    while iterations < MANDLEBULB_ITERATIONS {
        r = z.length();
        if r > 2.0 {
            break;
        };

        // Convert to polar coordinates
        let mut theta = (z.z / r).acos();
        let mut phi = z.y.atan2(z.x);
        dr = r.powf(power - 1.0) * power * dr + 1.0;

        // Scale and rotate the point
        let zr = r.powf(power);
        theta = theta * power;
        phi = phi * power;

        // Convert back to cartesian coordinates
        z = zr
            * vec3(
                theta.sin() * phi.cos(),
                theta.sin() * phi.sin(),
                theta.cos(),
            );
        z += pos;
        iterations += 1
    }

    let dist = MANDLEBULB_SCALE * r.log10() * r / dr;
    (dist, iterations as f64)
}

fn world(pos: Vec3, time: f64) -> f64 {
    let (dist, _) = mandlebulb_distance(pos, time);
    dist
}

fn normal_of_pos(pos: Vec3, time: f64) -> Vec3 {
    let step = vec3(0.001, 0.0, 0.0);

    let grad_x = world(pos + vec3(step.x, step.y, step.y), time)
        - world(pos - vec3(step.x, step.y, step.y), time);
    let grad_y = world(pos + vec3(step.y, step.x, step.y), time)
        - world(pos - vec3(step.y, step.x, step.y), time);
    let grad_z = world(pos + vec3(step.y, step.y, step.x), time)
        - world(pos - vec3(step.y, step.y, step.x), time);

    let normal = vec3(grad_x, grad_y, grad_z);

    -normal.normalize()
}

fn get_ray_dir(u: f64, v: f64, cam_pos: Vec3, look_at: Vec3, zoom: f64) -> Vec3 {
    let f = (look_at - cam_pos).normalize();
    let r = vec3(0.0, 1.0, 0.0).cross(&f);
    let u2 = f.cross(&r);
    let c = cam_pos + f * zoom;
    let i = c + u * r + v * u2;
    (i - cam_pos).normalize()
}

fn ray_march(opts: &RayMarchOpts) -> Vec3 {
    let mut total_dist_traveled = 0.0f64;
    let mut color = (vec3(1.4, 0.7, 10.1) + (opts.time / 7.0))
        .mix(vec3(opts.u, opts.u, opts.u), vec3(opts.v, opts.v, opts.v))
        * 0.015;
    let ray_dir = get_ray_dir(opts.u, opts.v, opts.cam_pos, opts.look_at, opts.zoom);

    let mut steps = 0;
    while steps < MAX_STEPS {
        let cur_pos = opts.cam_pos + total_dist_traveled * ray_dir;
        let (dist_to_closest, iterations) = mandlebulb_distance(cur_pos, opts.time);

        // Hit
        if dist_to_closest < MIN_HIT_DIST {
            let normal = normal_of_pos(cur_pos, opts.time);
            let light_pos = vec3(-4.0, -3.0, 0.0);
            let light_dir = (cur_pos - light_pos).normalize();
            let diffuse_intensity = (normal * 0.2 + 0.3).dot(&light_dir).clamp(0.0, 1.0);
            let color_a = (vec3(cur_pos.z, cur_pos.y, cur_pos.x + opts.time / 7.0)
                + vec3(1.4, 0.7, 10.1))
            .sin()
                * diffuse_intensity;
            let color_b = vec3(0.0, 0.0, 0.2) * (iterations / 16.0).clamp(0.0, 1.0);
            color = saturate(color_a + color_b);
            break;
        };

        // Miss
        if total_dist_traveled > MAX_TRACE_DIST {
            break;
        };

        total_dist_traveled += dist_to_closest;
        steps += 1;
    }

    // Inspired by https://github.com/SebLague/Ray-Marching/blob/f7e44c15a212dec53b244b1f53cdaf318f6ec700/Assets/Scripts/Fractal/Fractal.compute
    let rim = steps as f64 / DARKNESS;
    color.mix(vec3(0.6, 0.6, 0.6), vec3(0.03, 0.03, 0.03)) * rim
}

fn scene_fn(time: f64) -> Scene {
    Scene {
        nx: 1920,
        ny: 1080,
        ns: 10,
        time,
        cam_pos: vec3((time / 20.0) * 5.0, 0.0, -5.0),
        look_at: vec3(0.0, 0.0, 0.0),
        zoom: 2.0,
        ray_march_fn: Arc::new(&ray_march),
    }
}

fn main() {
    render_animation(
        AnimatedScene {
            fps: 30.0,
            start: 0.0,
            end: 60.0,
            scene_fn: &scene_fn,
        },
        "mandlebulb_animation".into(),
    );
}
