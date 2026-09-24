mod camera;
mod framebuffer;
mod material;
mod ray_intersect;
mod primitives;
mod scene;

use camera::Camera;
use framebuffer::Framebuffer;
use material::Material;
use ray_intersect::Ray;
use primitives::{Object, Plane, Cube, Sphere};
use scene::Scene;
use raylib::prelude::*;

const SCREEN_W: i32 = 800;
const SCREEN_H: i32 = 600;
const ROTATE_SPEED: f32 = 1.5;

const ROOM_W: f32 = 12.0;
const ROOM_H: f32 = 6.0;
const ROOM_D: f32 = 20.0;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_W, SCREEN_H)
        .title("Raytracer - Fase 1: Sistema unificado")
        .build();
    rl.set_target_fps(60);

    let mut fb = Framebuffer::new(SCREEN_W as usize, SCREEN_H as usize);
    let mut scene = Scene::new();

    // ===== Materiales =====
    let floor_mat   = Material::diffuse(Color::new(60, 60, 70, 255));
    let wall_mat    = Material::diffuse(Color::new(95, 95, 105, 255));
    let ceiling_mat = Material::diffuse(Color::new(210, 210, 215, 255));
    let screen_mat  = Material::new(Color::new(40, 80, 160, 255), 0.15, 0.7, 0.25, 64.0);

    // ===== Cuarto (6 planos) =====
    scene.add(Object::Plane(Plane::new(
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        floor_mat,
    )));

    scene.add(Object::Plane(Plane::new(
        Vector3::new(0.0, ROOM_H, 0.0),
        Vector3::new(0.0, -1.0, 0.0),
        ceiling_mat,
    )));

    scene.add(Object::Plane(Plane::new(
        Vector3::new(-ROOM_W / 2.0, 0.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        wall_mat,
    )));

    scene.add(Object::Plane(Plane::new(
        Vector3::new(ROOM_W / 2.0, 0.0, 0.0),
        Vector3::new(-1.0, 0.0, 0.0),
        wall_mat,
    )));

    let back_wall_idx = scene.add(Object::Plane(Plane::new(
        Vector3::new(0.0, 0.0, -ROOM_D / 2.0),
        Vector3::new(0.0, 0.0, 1.0),
        screen_mat,
    )));

    scene.add(Object::Plane(Plane::new(
        Vector3::new(0.0, 0.0, ROOM_D / 2.0),
        Vector3::new(0.0, 0.0, -1.0),
        wall_mat,
    )));

    // ===== Prueba de Fase 1: un cubo y una esfera dentro del cuarto =====
    scene.add(Object::Cube(Cube::new(
        Vector3::new(-2.0, 1.0, -4.0),
        Vector3::new(2.0, 2.0, 2.0),
        Material::diffuse(Color::new(220, 100, 50, 255)),
    )));

    scene.add(Object::Sphere(Sphere::new(
        Vector3::new(2.0, 1.0, -4.0),
        1.0,
        Material::diffuse(Color::new(80, 200, 100, 255)),
    )));

    // ===== Camara =====
    let mut camera = Camera::new(Vector3::new(0.0, 2.5, ROOM_D / 2.0 - 1.0));
    let fov: f32 = 60.0_f32.to_radians();
    let light_dir = Vector3::new(0.3, 0.9, 0.3).normalized();

    // ===== Materiales de prueba para el panel del fondo =====
    let panel_rojo      = Material::new(Color::new(200, 50, 50, 255), 0.15, 0.8, 0.2, 32.0);
    let panel_azul      = Material::new(Color::new(40, 80, 200, 255), 0.15, 0.8, 0.2, 32.0);
    let panel_verde     = Material::new(Color::new(50, 180, 90, 255), 0.15, 0.8, 0.2, 32.0);
    let panel_brillante = Material::new(Color::new(220, 220, 220, 255), 0.05, 0.4, 1.0, 128.0);

    let image = Image::gen_image_color(SCREEN_W, SCREEN_H, Color::BLACK);
    let mut texture = rl.load_texture_from_image(&thread, &image).unwrap();

    let mut needs_render = true;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

        let mut delta_yaw = 0.0;
        let mut delta_pitch = 0.0;
        if rl.is_key_down(KeyboardKey::KEY_LEFT)  { delta_yaw   -= ROTATE_SPEED * dt; }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) { delta_yaw   += ROTATE_SPEED * dt; }
        if rl.is_key_down(KeyboardKey::KEY_UP)    { delta_pitch += ROTATE_SPEED * dt; }
        if rl.is_key_down(KeyboardKey::KEY_DOWN)  { delta_pitch -= ROTATE_SPEED * dt; }
        if delta_yaw != 0.0 || delta_pitch != 0.0 {
            camera.rotate(delta_yaw, delta_pitch);
            needs_render = true;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_ONE) {
            scene.update_material(back_wall_idx, panel_rojo);
            needs_render = true;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_TWO) {
            scene.update_material(back_wall_idx, panel_azul);
            needs_render = true;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_THREE) {
            scene.update_material(back_wall_idx, panel_verde);
            needs_render = true;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_FOUR) {
            scene.update_material(back_wall_idx, panel_brillante);
            needs_render = true;
        }

        if needs_render {
            render(&mut fb, &scene, &camera, fov, light_dir);
            texture.update_texture(&fb.pixels).unwrap();
            needs_render = false;
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&texture, 0, 0, Color::WHITE);
        d.draw_fps(10, 10);
        d.draw_text("Flechas = rotar camara", 10, SCREEN_H - 45, 16, Color::WHITE);
        d.draw_text("1/2/3/4 = color del panel del fondo", 10, SCREEN_H - 25, 16, Color::WHITE);
    }
}

fn render(fb: &mut Framebuffer, scene: &Scene, camera: &Camera, fov: f32, light_dir: Vector3) {
    let width = fb.width as f32;
    let height = fb.height as f32;
    let aspect_ratio = width / height;
    let tan_half_fov = (fov / 2.0).tan();

    let (forward, right, up) = camera.basis();

    for y in 0..fb.height {
        for x in 0..fb.width {
            let screen_x = (2.0 * (x as f32 + 0.5) / width - 1.0) * aspect_ratio * tan_half_fov;
            let screen_y = (1.0 - 2.0 * (y as f32 + 0.5) / height) * tan_half_fov;

            let direction = forward + right * screen_x + up * screen_y;
            let ray = Ray::new(camera.eye, direction);

            let color = match scene.closest_hit(&ray) {
                Some((idx, hit)) => {
                    let view_dir = (camera.eye - hit.point).normalized();
                    scene.objects[idx].material().shade(hit.normal, light_dir, view_dir)
                }
                None => Color::new(15, 15, 20, 255),
            };

            fb.set_pixel(x, y, color);
        }
    }
}