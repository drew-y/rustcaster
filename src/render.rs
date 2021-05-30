use super::scene::{AnimatedScene, RayMarchOpts, Scene};
use super::vec3::vec3;
use image;
use indicatif::{ProgressBar, ProgressStyle};
use num_cpus;
use rand::prelude::*;
use std::thread;

fn render_section(scene: Scene, start_y: i32, end_y: i32, pb: ProgressBar) -> Vec<u8> {
    let mut file: Vec<u8> = Vec::with_capacity((end_y - start_y) as usize * scene.nx as usize * 3);
    let mut rng = thread_rng();
    let Scene { nx, ny, ns, .. } = scene;

    for j in (start_y..end_y).rev() {
        for i in 0..nx {
            let mut col = vec3(0.0, 0.0, 0.0);
            for _s in 0..ns {
                let u = (i as f64 + rng.gen::<f64>()) / nx as f64 - 0.5;
                let v = (j as f64 + rng.gen::<f64>()) / ny as f64 - 0.5;
                col += (scene.ray_march_fn)(&RayMarchOpts {
                    u,
                    v,
                    ns: scene.ns,
                    time: scene.time,
                    cam_pos: scene.cam_pos,
                    look_at: scene.look_at,
                    zoom: scene.zoom,
                });
            }
            col /= ns as f64;
            file.push((255.99f64 * col.x).max(0.0).min(255.0) as u8);
            file.push((255.99f64 * col.y).max(0.0).min(255.0) as u8);
            file.push((255.99f64 * col.z).max(0.0).min(255.0) as u8);
        }
        pb.inc(1);
    }
    file
}

fn render_progress_bar(ny: i32) -> ProgressBar {
    let pb = ProgressBar::new(ny as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {msg} [{bar:50.cyan/blue}] ({eta})")
            .progress_chars("#>-"),
    );
    pb.set_message("Rendering");
    pb.set_position(0);
    pb
}

pub fn render(scene: Scene, path: String) {
    let Scene { nx, ny, .. } = scene;
    let mut file: Vec<u8> = Vec::with_capacity((nx as usize) * (ny as usize) * 3);

    let thread_count = num_cpus::get();
    let mut render_threads: Vec<thread::JoinHandle<Vec<u8>>> = Vec::with_capacity(thread_count);
    let y_section_size = ny / thread_count as i32;
    let mut start_y = ny - y_section_size;
    let mut end_y = ny;
    let pb = render_progress_bar(ny);

    for _thread in 0..thread_count {
        let thread_scene = scene.clone();
        let thread_pb = pb.clone();
        let render_thread =
            thread::spawn(move || render_section(thread_scene, start_y, end_y, thread_pb));
        render_threads.push(render_thread);
        end_y = start_y;
        start_y -= y_section_size;
    }

    for render_thread in render_threads {
        file.extend(render_thread.join().unwrap());
    }

    // Render remaining y columns
    let remaining_y_columns = ny - (y_section_size * thread_count as i32);
    if remaining_y_columns > 0 {
        render_threads = Vec::with_capacity(remaining_y_columns as usize);
        for column in (0..remaining_y_columns).rev() {
            let thread_scene = scene.clone();
            let thread_pb = pb.clone();
            let render_thread =
                thread::spawn(move || render_section(thread_scene, column - 1, column, thread_pb));
            render_threads.push(render_thread);
        }

        for render_thread in render_threads {
            file.extend(render_thread.join().unwrap());
        }
    }

    pb.finish_with_message("Complete");

    match image::save_buffer(path, &file, nx as u32, ny as u32, image::ColorType::RGB(8)) {
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        _ => {}
    }
}

pub fn render_animation(scene: AnimatedScene, path: String) {
    let time_step = 1.0 / scene.fps;
    let mut time = scene.start;

    let mut frame = (time / time_step) as i32 + 1;
    while time <= scene.end {
        render(
            (scene.scene_fn)(time),
            format!("./{}/frame-{}.png", path, frame),
        );
        time += time_step;
        frame += 1;
    }
}
