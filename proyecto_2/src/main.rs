mod camera;
mod framebuffer;
mod material;
mod ray_intersect;
mod light;
mod primitives;
mod scene;

use camera::Camera;
use framebuffer::Framebuffer;
use material::Material;
use ray_intersect::Ray;
use light::Light;
use primitives::{Object, Plane, Cube, Sphere};
use scene::Scene;
use raylib::prelude::*;

// ============ CONFIG ============
const SCREEN_W: i32 = 800;
const SCREEN_H: i32 = 600;
const RENDER_SCALE: f32 = 0.5;

const ROTATE_SPEED: f32 = 1.5;
const SUN_SPEED: f32 = 0.6;

const ROOM_W: f32 = 12.0;
const ROOM_H: f32 = 6.0;
const ROOM_D: f32 = 20.0;
const STAGE_HEIGHT: f32 = 1.0;
const STAGE_DEPTH: f32 = 7.0;

const NUM_STEPS: usize = 6;
const STEP_RISE: f32 = 0.4;
const STEP_DEPTH: f32 = 1.5;
const STEPS_START_Z: f32 = -2.0;
const CHAIRS_PER_ROW: usize = 6;
const CHAIR_GAP: f32 = 1.6;

const NUM_LIGHT_ROWS: usize = 4;
const NUM_LIGHT_COLS: usize = 3;

// ============ SELECCIÓN DE MATERIAL ============
#[derive(Clone, Copy, PartialEq)]
enum MaterialKind {
    Diffuse, Glossy, Plastic, Metal, Emissive, Mirror,
}

impl MaterialKind {
    fn build(&self, color: Color) -> Material {
        match self {
            MaterialKind::Diffuse  => Material::new(color, 0.15, 0.85, 0.0, 1.0),
            MaterialKind::Glossy   => Material::new(color, 0.10, 0.70, 0.5, 64.0),
            MaterialKind::Plastic  => Material::new(color, 0.20, 0.70, 0.3, 32.0),
            MaterialKind::Metal    => Material::new(color, 0.05, 0.35, 1.0, 128.0),
            MaterialKind::Emissive => Material::emissive(color),
            MaterialKind::Mirror   => Material::new(color, 0.05, 0.30, 1.0, 256.0),
        }
    }
    fn name(&self) -> &'static str {
        match self {
            MaterialKind::Diffuse  => "Difuso",
            MaterialKind::Glossy   => "Brillante",
            MaterialKind::Plastic  => "Plastico",
            MaterialKind::Metal    => "Metal",
            MaterialKind::Emissive => "Emisivo",
            MaterialKind::Mirror   => "Espejo",
        }
    }
}

fn color_by_index(i: usize) -> Color {
    match i {
        0 => Color::new(220,  60,  60, 255),
        1 => Color::new(240, 140,  50, 255),
        2 => Color::new(240, 220,  60, 255),
        3 => Color::new( 80, 210, 100, 255),
        4 => Color::new( 70, 200, 220, 255),
        5 => Color::new( 70, 110, 230, 255),
        6 => Color::new(160,  80, 220, 255),
        7 => Color::new(230, 100, 180, 255),
        8 => Color::new(230, 230, 235, 255),
        _ => Color::WHITE,
    }
}

// ============ HANDLES DINÁMICOS ============
struct AuditoriumHandles {
    light_objs:     Vec<(usize, usize)>,
    window_indices: Vec<usize>,
    star_indices:   Vec<usize>,
}

// ============ PINTURA SECÁNDOSE ============
struct DryingPaint {
    obj_idx: usize,
    wet_mat: Material,
    dry_mat: Material,
    elapsed: f32,
    duration: f32,
}

/// Devuelve (duración_base, factor_de_humedad) para cada tipo de material.
/// - duración: cuánto tarda en secar (segundos).
/// - humedad:  qué tanto se diferencia el estado fresco del seco (0..1).
fn drying_params(kind: MaterialKind) -> (f32, f32) {
    match kind {
        MaterialKind::Mirror   => (1.5, 0.15),
        MaterialKind::Metal    => (2.0, 0.25),
        MaterialKind::Emissive => (3.0, 0.15),
        MaterialKind::Glossy   => (2.5, 0.40),
        MaterialKind::Plastic  => (3.5, 0.60),
        MaterialKind::Diffuse  => (4.5, 0.85),
    }
}

/// Genera el material "fresco" a partir del seco y un factor de humedad.
/// La pintura fresca es: más brillante (specular alto), con highlight más
/// concentrado (shininess alto), y un albedo un poco más oscuro (como cuando
/// se moja un material).
fn make_wet_material(dry: &Material, wetness: f32) -> Material {
    let specular = (dry.specular + 0.6 * wetness).min(1.5);
    let shininess = dry.shininess + (128.0 - dry.shininess).max(0.0) * wetness;

    let darken = 1.0 - 0.20 * wetness;
    let albedo = Color::new(
        (dry.albedo.r as f32 * darken) as u8,
        (dry.albedo.g as f32 * darken) as u8,
        (dry.albedo.b as f32 * darken) as u8,
        255,
    );

    Material {
        albedo,
        ambient: dry.ambient,
        diffuse: dry.diffuse * (1.0 - 0.15 * wetness),
        specular,
        shininess,
        emissive: dry.emissive,
    }
}

// ============ INTERPOLACIONES ============
fn lerp(a: f32, b: f32, t: f32) -> f32 { a + (b - a) * t }

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        lerp(a.r as f32, b.r as f32, t) as u8,
        lerp(a.g as f32, b.g as f32, t) as u8,
        lerp(a.b as f32, b.b as f32, t) as u8,
        255,
    )
}

fn lerp_material(a: &Material, b: &Material, t: f32) -> Material {
    Material {
        albedo:    lerp_color(a.albedo, b.albedo, t),
        ambient:   lerp(a.ambient,   b.ambient,   t),
        diffuse:   lerp(a.diffuse,   b.diffuse,   t),
        specular:  lerp(a.specular,  b.specular,  t),
        shininess: lerp(a.shininess, b.shininess, t),
        emissive:  lerp(a.emissive,  b.emissive,  t),
    }
}

// ============ MAIN ============
fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_W, SCREEN_H)
        .title("Raytracer - Auditorio ITESO (interactivo)")
        .build();
    rl.set_target_fps(60);

    let render_w = (SCREEN_W as f32 * RENDER_SCALE) as usize;
    let render_h = (SCREEN_H as f32 * RENDER_SCALE) as usize;
    let mut fb = Framebuffer::new(render_w, render_h);

    let mut scene = Scene::new(NUM_LIGHT_ROWS);
    let handles = build_auditorium(&mut scene);

    let mut camera = Camera::new(Vector3::new(0.0, 4.0, 8.5));
    let fov: f32 = 60.0_f32.to_radians();

    let image = Image::gen_image_color(render_w as i32, render_h as i32, Color::BLACK);
    let mut texture = rl.load_texture_from_image(&thread, &image).unwrap();
    texture.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);

    let mut needs_render = true;

    let mut paint_color_idx: usize = 0;
    let mut paint_kind = MaterialKind::Diffuse;

    let mut sun_t: f32 = 1.0;
    let mut target_sun_t: f32 = 1.0;
    let mut last_is_day: bool = true;

    update_day_night(&mut scene, &handles, sun_t);

    let mut hover_hit = false;

    // Lista de pinturas que están secándose
    let mut drying: Vec<DryingPaint> = Vec::new();

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

        // ============ CÁMARA ============
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

        // ============ COLORES 1-9 ============
        let color_keys = [
            KeyboardKey::KEY_ONE,   KeyboardKey::KEY_TWO,   KeyboardKey::KEY_THREE,
            KeyboardKey::KEY_FOUR,  KeyboardKey::KEY_FIVE,  KeyboardKey::KEY_SIX,
            KeyboardKey::KEY_SEVEN, KeyboardKey::KEY_EIGHT, KeyboardKey::KEY_NINE,
        ];
        for (i, key) in color_keys.iter().enumerate() {
            if rl.is_key_pressed(*key) { paint_color_idx = i; }
        }

        // ============ MATERIALES QWERTY ============
        if rl.is_key_pressed(KeyboardKey::KEY_Q) { paint_kind = MaterialKind::Diffuse; }
        if rl.is_key_pressed(KeyboardKey::KEY_W) { paint_kind = MaterialKind::Glossy; }
        if rl.is_key_pressed(KeyboardKey::KEY_E) { paint_kind = MaterialKind::Plastic; }
        if rl.is_key_pressed(KeyboardKey::KEY_R) { paint_kind = MaterialKind::Metal; }
        if rl.is_key_pressed(KeyboardKey::KEY_T) { paint_kind = MaterialKind::Emissive; }
        if rl.is_key_pressed(KeyboardKey::KEY_Y) { paint_kind = MaterialKind::Mirror; }

        // ============ PINTAR CON EL MOUSE ============
        let mouse = rl.get_mouse_position();
        let paint_ray = ray_from_screen(mouse, SCREEN_W as f32, SCREEN_H as f32, fov, &camera);
        hover_hit = scene.closest_hit(&paint_ray).is_some();

        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) && hover_hit {
            if let Some((idx, _)) = scene.closest_hit(&paint_ray) {
                let dry_mat = paint_kind.build(color_by_index(paint_color_idx));
                let (base_duration, wetness) = drying_params(paint_kind);

                // El color afecta: colores más oscuros tardan más en secar
                let brightness = (dry_mat.albedo.r as f32
                    + dry_mat.albedo.g as f32
                    + dry_mat.albedo.b as f32) / (255.0 * 3.0);
                let color_factor = 1.0 + (1.0 - brightness) * 0.6;

                let duration = base_duration * color_factor;
                let wet_mat = make_wet_material(&dry_mat, wetness);

                // Aplicar el material húmedo inmediatamente
                scene.update_material(idx, wet_mat);

                // Registrar/refrescar el secado de este objeto
                drying.retain(|d| d.obj_idx != idx);
                drying.push(DryingPaint {
                    obj_idx: idx,
                    wet_mat,
                    dry_mat,
                    elapsed: 0.0,
                    duration,
                });

                needs_render = true;
            }
        }

        // ============ LUCES A S D F / G ============
        if rl.is_key_pressed(KeyboardKey::KEY_A) { scene.toggle_row(0); update_light_visuals(&mut scene, &handles.light_objs); needs_render = true; }
        if rl.is_key_pressed(KeyboardKey::KEY_S) { scene.toggle_row(1); update_light_visuals(&mut scene, &handles.light_objs); needs_render = true; }
        if rl.is_key_pressed(KeyboardKey::KEY_D) { scene.toggle_row(2); update_light_visuals(&mut scene, &handles.light_objs); needs_render = true; }
        if rl.is_key_pressed(KeyboardKey::KEY_F) { scene.toggle_row(3); update_light_visuals(&mut scene, &handles.light_objs); needs_render = true; }
        if rl.is_key_pressed(KeyboardKey::KEY_G) { scene.toggle_all();         update_light_visuals(&mut scene, &handles.light_objs); needs_render = true; }

        // ============ DÍA / NOCHE ============
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            target_sun_t = if target_sun_t > 0.5 { 0.0 } else { 1.0 };
        }
        let diff = target_sun_t - sun_t;
        let step = SUN_SPEED * dt;
        if diff.abs() <= step {
            if sun_t != target_sun_t {
                sun_t = target_sun_t;
                needs_render = true;
            }
        } else {
            sun_t += diff.signum() * step;
            needs_render = true;
        }

        let is_day = sun_t > 0.5;
        if is_day != last_is_day {
            update_day_night(&mut scene, &handles, sun_t);
            last_is_day = is_day;
            needs_render = true;
        }

        // ============ ACTUALIZAR PINTURA SECÁNDOSE ============
        if !drying.is_empty() {
            for d in drying.iter_mut() {
                d.elapsed = (d.elapsed + dt).min(d.duration);
                let x = (d.elapsed / d.duration).clamp(0.0, 1.0);
                // Ease-out: rápido al principio, lento al final
                let t = 1.0 - (1.0 - x) * (1.0 - x);
                let mat = lerp_material(&d.wet_mat, &d.dry_mat, t);
                scene.update_material(d.obj_idx, mat);
            }
            drying.retain(|d| d.elapsed < d.duration);
            needs_render = true;
        }

        // ============ RENDER ============
        if needs_render {
            render(&mut fb, &scene, &camera, fov, sun_t);
            texture.update_texture(&fb.pixels).unwrap();
            needs_render = false;
        }

        // ============ DIBUJADO ============
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        d.draw_texture_pro(
            &texture,
            Rectangle { x: 0.0, y: 0.0, width: render_w as f32, height: render_h as f32 },
            Rectangle { x: 0.0, y: 0.0, width: SCREEN_W as f32, height: SCREEN_H as f32 },
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );

        let cx = mouse.x as i32;
        let cy = mouse.y as i32;
        let cursor_color = if hover_hit { Color::GREEN } else { Color::WHITE };
        d.draw_rectangle_lines(cx - 8, cy - 8, 16, 16, cursor_color);

        d.draw_fps(10, 10);

        let info_color = color_by_index(paint_color_idx);
        d.draw_rectangle(10, 30, 20, 20, info_color);
        d.draw_rectangle_lines(10, 30, 20, 20, Color::WHITE);
        d.draw_text(&format!("Material: {}", paint_kind.name()), 38, 32, 16, Color::WHITE);

        let day_phase = if sun_t > 0.5 { "DIA" } else { "NOCHE" };
        d.draw_text(&format!("Sol: {} ({:.2})", day_phase, sun_t), 10, 55, 16, Color::YELLOW);

        // Indicador de pinturas secándose
        if !drying.is_empty() {
            d.draw_text(&format!("Secando {} zona(s)...", drying.len()),
                        10, 78, 16, Color::ORANGE);
        }

        d.draw_text("Flechas: camara | Click: pintar | 1-9: color | QWERTY: material",
                    10, SCREEN_H - 65, 14, Color::WHITE);
        d.draw_text("A/S/D/F: filas de luces | G: todas | ESPACIO: dia/noche",
                    10, SCREEN_H - 45, 14, Color::WHITE);

        let row_state: String = (0..NUM_LIGHT_ROWS)
            .map(|i| if scene.light_row_enabled[i] { "●" } else { "○" })
            .collect::<Vec<_>>()
            .join(" ");
        d.draw_text(&format!("Filas de luces: {}", row_state), 10, SCREEN_H - 25, 14, Color::SKYBLUE);
    }
}

// ============ RAYO DESDE EL MOUSE ============
fn ray_from_screen(mouse: Vector2, w: f32, h: f32, fov: f32, camera: &Camera) -> Ray {
    let aspect = w / h;
    let tan_half = (fov / 2.0).tan();
    let sx = (2.0 * mouse.x / w - 1.0) * aspect * tan_half;
    let sy = (1.0 - 2.0 * mouse.y / h) * tan_half;
    let (forward, right, up) = camera.basis();
    let dir = forward + right * sx + up * sy;
    Ray::new(camera.eye, dir)
}

// ============ LUCES: VISUAL ON/OFF ============
fn update_light_visuals(scene: &mut Scene, light_objs: &[(usize, usize)]) {
    for (row, obj_idx) in light_objs.iter() {
        let on = scene.light_row_enabled[*row];
        let mat = if on {
            Material::emissive(Color::new(255, 250, 230, 255))
        } else {
            Material::new(Color::new(60, 60, 70, 255), 0.1, 0.5, 0.0, 1.0)
        };
        scene.update_material(*obj_idx, mat);
    }
}

// ============ VENTANAS / ESTRELLAS ============
fn update_day_night(scene: &mut Scene, handles: &AuditoriumHandles, sun_t: f32) {
    let day = sun_t > 0.5;

    let sky_mat = if day {
        Material::emissive(Color::new(135, 180, 230, 255))
    } else {
        Material::new(Color::new(5, 8, 20, 255), 0.05, 0.15, 0.0, 1.0)
    };
    for &idx in &handles.window_indices {
        scene.update_material(idx, sky_mat);
    }

    let star_mat = if day {
        Material::emissive(Color::new(135, 180, 230, 255))
    } else {
        Material::emissive(Color::new(255, 255, 255, 255))
    };
    for &idx in &handles.star_indices {
        scene.update_material(idx, star_mat);
    }
}

// ============ CONSTRUCCIÓN DEL AUDITORIO ============
fn build_auditorium(scene: &mut Scene) -> AuditoriumHandles {
    let floor_mat     = Material::diffuse(Color::new(45, 45, 55, 255));
    let wall_mat      = Material::diffuse(Color::new(70, 70, 80, 255));
    let ceiling_mat   = Material::diffuse(Color::new(200, 200, 210, 255));
    let stage_mat     = Material::new(Color::new(165, 120, 75, 255), 0.15, 0.8, 0.15, 24.0);
    let screen_mat    = Material::new(Color::new(40, 80, 160, 255),  0.15, 0.7, 0.25, 64.0);
    let wood_mat      = Material::diffuse(Color::new(180, 140, 90, 255));
    let dark_wood_mat = Material::diffuse(Color::new(90, 60, 40, 255));
    let podium_mat    = Material::diffuse(Color::new(110, 80, 55, 255));
    let chair_mat     = Material::diffuse(Color::new(30, 40, 70, 255));
    let step_mat      = Material::diffuse(Color::new(65, 65, 75, 255));
    let light_mat_on  = Material::emissive(Color::new(255, 250, 230, 255));

    // Cuarto
    scene.add(Object::Plane(Plane::new(Vector3::new(0.0, 0.0, 0.0),  Vector3::new(0.0, 1.0, 0.0),  floor_mat)));
    scene.add(Object::Plane(Plane::new(Vector3::new(0.0, ROOM_H, 0.0), Vector3::new(0.0,-1.0, 0.0),  ceiling_mat)));
    scene.add(Object::Plane(Plane::new(Vector3::new(-ROOM_W/2.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0), wall_mat)));
    scene.add(Object::Plane(Plane::new(Vector3::new( ROOM_W/2.0, 0.0, 0.0), Vector3::new(-1.0,0.0, 0.0), wall_mat)));
    scene.add(Object::Plane(Plane::new(Vector3::new(0.0, 0.0, -ROOM_D/2.0), Vector3::new(0.0, 0.0, 1.0), wall_mat)));
    scene.add(Object::Plane(Plane::new(Vector3::new(0.0, 0.0,  ROOM_D/2.0), Vector3::new(0.0, 0.0,-1.0), wall_mat)));

    // Escenario
    scene.add(Object::Cube(Cube::new(
        Vector3::new(0.0, STAGE_HEIGHT/2.0, -ROOM_D/2.0 + STAGE_DEPTH/2.0),
        Vector3::new(11.5, STAGE_HEIGHT, STAGE_DEPTH),
        stage_mat,
    )));

    // Panel del fondo
    scene.add(Object::Cube(Cube::new(
        Vector3::new(0.0, 3.0, -ROOM_D/2.0 + 0.3),
        Vector3::new(5.0, 3.5, 0.4),
        screen_mat,
    )));

    // Slats de madera
    let num_slats = 14;
    let y_min = 1.5; let y_max = 5.3;
    let step = (y_max - y_min) / num_slats as f32;
    let slat_height = step * 0.7;
    for i in 0..num_slats {
        let y = y_min + (i as f32 + 0.5) * step;
        scene.add(Object::Cube(Cube::new(Vector3::new(-4.25, y, -ROOM_D/2.0 + 0.2),
            Vector3::new(3.5, slat_height, 0.15), wood_mat)));
        scene.add(Object::Cube(Cube::new(Vector3::new(4.25, y, -ROOM_D/2.0 + 0.2),
            Vector3::new(3.5, slat_height, 0.15), wood_mat)));
    }

    // Podium
    scene.add(Object::Cube(Cube::new(
        Vector3::new(-2.8, STAGE_HEIGHT + 0.55, -ROOM_D/2.0 + 3.5),
        Vector3::new(0.7, 1.1, 0.5),
        podium_mat,
    )));

    // Mesa presídium
    scene.add(Object::Cube(Cube::new(
        Vector3::new(1.0, STAGE_HEIGHT + 0.4, -ROOM_D/2.0 + 3.2),
        Vector3::new(5.0, 0.8, 0.8),
        dark_wood_mat,
    )));

    // Gradería
    for i in 0..NUM_STEPS {
        let top_y = (i as f32 + 1.0) * STEP_RISE;
        let bot_y = i as f32 * STEP_RISE;
        let z_center = STEPS_START_Z + (i as f32 + 0.5) * STEP_DEPTH;
        scene.add(Object::Cube(Cube::new(
            Vector3::new(0.0, (top_y + bot_y) / 2.0, z_center),
            Vector3::new(11.5, STEP_RISE, STEP_DEPTH),
            step_mat,
        )));
    }

    // Sillas
    let total_width = (CHAIRS_PER_ROW - 1) as f32 * CHAIR_GAP;
    let start_x = -total_width / 2.0;
    for i in 0..NUM_STEPS {
        let top_y = (i as f32 + 1.0) * STEP_RISE;
        let z_center = STEPS_START_Z + (i as f32 + 0.5) * STEP_DEPTH;
        for c in 0..CHAIRS_PER_ROW {
            let x = start_x + c as f32 * CHAIR_GAP;
            scene.add(Object::Cube(Cube::new(Vector3::new(x, top_y + 0.05, z_center),
                Vector3::new(0.5, 0.1, 0.5), chair_mat)));
            scene.add(Object::Cube(Cube::new(Vector3::new(x, top_y + 0.35, z_center + 0.2),
                Vector3::new(0.5, 0.5, 0.1), chair_mat)));
        }
    }

    // Ventanas + estrellas
    let window_zs = [-8.0, -5.0, -2.0, 1.0, 4.0, 7.0];
    let day_window_mat = Material::emissive(Color::new(135, 180, 230, 255));
    let day_star_mat   = Material::emissive(Color::new(135, 180, 230, 255));

    let mut window_indices: Vec<usize> = Vec::new();
    let mut star_indices:   Vec<usize> = Vec::new();

    let star_pattern: [(f32, f32); 6] = [
        ( 0.9,  0.10), ( 0.3, -0.12), (-0.4,  0.05),
        ( 1.2, -0.08), (-1.0,  0.08), (-0.2, -0.14),
    ];

    for z in window_zs {
        let wl = scene.add(Object::Cube(Cube::new(
            Vector3::new(-ROOM_W/2.0 + 0.05, 3.0, z),
            Vector3::new(0.1, 5.0, 0.4),
            day_window_mat,
        )));
        window_indices.push(wl);
        for (dy, dz) in star_pattern {
            let s = scene.add(Object::Sphere(Sphere::new(
                Vector3::new(-ROOM_W/2.0 + 0.15, 3.0 + dy, z + dz),
                0.04,
                day_star_mat,
            )));
            star_indices.push(s);
        }

        let wr = scene.add(Object::Cube(Cube::new(
            Vector3::new(ROOM_W/2.0 - 0.05, 3.0, z),
            Vector3::new(0.1, 5.0, 0.4),
            day_window_mat,
        )));
        window_indices.push(wr);
        for (dy, dz) in star_pattern {
            let s = scene.add(Object::Sphere(Sphere::new(
                Vector3::new(ROOM_W/2.0 - 0.15, 3.0 + dy, z + dz),
                0.04,
                day_star_mat,
            )));
            star_indices.push(s);
        }
    }

    // Focos del techo
    let mut light_objs: Vec<(usize, usize)> = Vec::new();
    for row in 0..NUM_LIGHT_ROWS {
        for col in 0..NUM_LIGHT_COLS {
            let x = -3.5 + col as f32 * 3.5;
            let z = -8.0 + row as f32 * 4.0;
            let pos = Vector3::new(x, ROOM_H - 0.2, z);
            let obj_idx = scene.add(Object::Sphere(Sphere::new(pos, 0.13, light_mat_on)));
            let row_idx = NUM_LIGHT_ROWS - 1 - row;
            scene.add_light(Light::new(pos, Color::new(255, 245, 220, 255), 6.0, row_idx));
            light_objs.push((row_idx, obj_idx));
        }
    }

    AuditoriumHandles { light_objs, window_indices, star_indices }
}

// ============ RENDER ============
fn render(fb: &mut Framebuffer, scene: &Scene, camera: &Camera, fov: f32, sun_t: f32) {
    let width = fb.width as f32;
    let height = fb.height as f32;
    let aspect_ratio = width / height;
    let tan_half_fov = (fov / 2.0).tan();

    let (forward, right, up) = camera.basis();

    let elevation = sun_t * (std::f32::consts::PI / 2.0);
    let sun_dir = Vector3::new(0.25, elevation.sin(), -elevation.cos()).normalized();
    let sun_intensity = (sun_t * sun_t) * 1.2;

    let sun_color = if sun_t > 0.75 {
        Color::new(255, 245, 220, 255)
    } else {
        let t = (sun_t / 0.75).clamp(0.0, 1.0);
        Color::new(
            255,
            (110.0 + 135.0 * t) as u8,
            (60.0  + 160.0 * t) as u8,
            255,
        )
    };

    let ambient_scale = 0.06 + 0.24 * sun_t;

    let bg = {
        let r = (6.0  + 30.0  * sun_t) as u8;
        let g = (10.0 + 55.0  * sun_t) as u8;
        let b = (22.0 + 100.0 * sun_t) as u8;
        Color::new(r, g, b, 255)
    };

    for y in 0..fb.height {
        for x in 0..fb.width {
            let sx = (2.0 * (x as f32 + 0.5) / width - 1.0) * aspect_ratio * tan_half_fov;
            let sy = (1.0 - 2.0 * (y as f32 + 0.5) / height) * tan_half_fov;

            let direction = forward + right * sx + up * sy;
            let ray = Ray::new(camera.eye, direction);

            let color = match scene.closest_hit(&ray) {
                Some((idx, hit)) => {
                    let view_dir = (camera.eye - hit.point).normalized();
                    scene.shade(
                        &scene.objects[idx].material(),
                        &hit,
                        view_dir,
                        sun_dir,
                        sun_color,
                        sun_intensity,
                        ambient_scale,
                    )
                }
                None => bg,
            };

            fb.set_pixel(x, y, color);
        }
    }
}