use crate::vec3::Vec3;
use std::sync::Arc;

#[derive(Clone)]
pub struct RayMarchOpts {
    pub u: f64,
    pub v: f64,
    pub ns: i32,
    pub time: f64,
    pub cam_pos: Vec3,
    pub look_at: Vec3,
    pub zoom: f64,
}

/// Returns a color
pub type RayMarchFn = Arc<dyn Fn(&RayMarchOpts) -> Vec3 + Sync + Send>;

#[derive(Clone)]
pub struct Scene {
    pub nx: i32,
    pub ny: i32,
    /// Sample count
    pub ns: i32,
    pub time: f64,
    pub cam_pos: Vec3,
    pub look_at: Vec3,
    pub zoom: f64,
    pub ray_march_fn: RayMarchFn,
}

/// Spits out a scene given a time
pub type SceneFn = &'static dyn Fn(f64) -> Scene;

#[derive(Clone)]
pub struct AnimatedScene {
    pub fps: f64,
    pub start: f64,
    pub end: f64,
    pub scene_fn: SceneFn,
}
