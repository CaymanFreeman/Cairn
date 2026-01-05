use glam::f32::Vec3;
use wgpu::util::DeviceExt as _;
use winit::keyboard::KeyCode;

const CAMERA_STARTING_POSITION: Vec3 = Vec3::new(0.0, 12.0, 0.0);
const CAMERA_FOV_Y: f32 = 90.0;
const CAMERA_Z_NEAR: f32 = 0.1;
const CAMERA_Z_FAR: f32 = 100.0;
const CAMERA_MOVE_SPEED: f32 = 0.03;
const CAMERA_MOVE_SPEED_SHIFT_MULTIPLIER: f32 = 3.5;
const CAMERA_TURN_SPEED: f32 = 0.02;
const CAMERA_MAX_PITCH: f32 = f32::to_radians(89.9);
const MOUSE_SENSITIVITY: f32 = 0.02;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_projection: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new(view_projection: [[f32; 4]; 4]) -> Self {
        Self { view_projection }
    }
}

pub(crate) struct Camera {
    position: Vec3,
    yaw: f32,
    pitch: f32,
    aspect_ratio: f32,
    uniform: CameraUniform,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl Camera {
    pub(crate) fn new(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let position = CAMERA_STARTING_POSITION;
        let yaw: f32 = 0.0;
        let pitch: f32 = 0.0;

        let aspect_ratio = surface_config.width as f32 / surface_config.height as f32;

        let forward = Vec3::new(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
        )
        .normalize();
        let target = position + forward;
        let view_matrix = glam::Mat4::look_at_rh(position, target, Vec3::Y);
        let projection =
            glam::Mat4::perspective_rh(CAMERA_FOV_Y, aspect_ratio, CAMERA_Z_NEAR, CAMERA_Z_FAR);
        let view_projection = projection * view_matrix;

        let uniform = CameraUniform::new(view_projection.to_cols_array_2d());

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        Self {
            position,
            yaw,
            pitch,
            aspect_ratio,
            uniform,
            buffer,
            bind_group,
        }
    }

    pub(crate) fn bind_group(&self) -> wgpu::BindGroup {
        self.bind_group.clone()
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.aspect_ratio = width as f32 / height as f32;
        self.update_view_projection();
    }

    fn view_matrix(&self) -> glam::Mat4 {
        let target = self.position + self.forward();
        glam::Mat4::look_at_rh(self.position, target, Vec3::Y)
    }

    fn projection(&self) -> glam::Mat4 {
        glam::Mat4::perspective_rh(CAMERA_FOV_Y, self.aspect_ratio, CAMERA_Z_NEAR, CAMERA_Z_FAR)
    }

    fn forward(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize()
    }

    fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y).normalize()
    }

    fn update_view_projection(&mut self) {
        self.uniform.view_projection = (self.projection() * self.view_matrix()).to_cols_array_2d();
    }

    pub(crate) fn update_buffer(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }
}

pub(crate) struct CameraController {
    mouse_sensitivity: f32,
    mouse_delta: (f32, f32),
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_sprint_pressed: bool,
    is_turn_left_pressed: bool,
    is_turn_right_pressed: bool,
    is_turn_up_pressed: bool,
    is_turn_down_pressed: bool,
}

impl CameraController {
    pub(crate) fn new() -> Self {
        Self {
            mouse_sensitivity: MOUSE_SENSITIVITY,
            mouse_delta: (0.0, 0.0),
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
            is_sprint_pressed: false,
            is_turn_left_pressed: false,
            is_turn_right_pressed: false,
            is_turn_up_pressed: false,
            is_turn_down_pressed: false,
        }
    }

    pub(crate) fn handle_mouse_input(&mut self, delta_x: f32, delta_y: f32) {
        self.mouse_delta = (delta_x, delta_y);
    }

    pub(crate) fn handle_keyboard_input(&mut self, code: KeyCode, is_pressed: bool) -> bool {
        match code {
            KeyCode::KeyW => {
                self.is_forward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyS => {
                self.is_backward_pressed = is_pressed;
                true
            }
            KeyCode::KeyD => {
                self.is_right_pressed = is_pressed;
                true
            }
            KeyCode::Space => {
                self.is_up_pressed = is_pressed;
                true
            }
            KeyCode::ControlLeft => {
                self.is_down_pressed = is_pressed;
                true
            }
            KeyCode::ShiftLeft => {
                self.is_sprint_pressed = is_pressed;
                true
            }
            KeyCode::ArrowLeft => {
                self.is_turn_left_pressed = is_pressed;
                true
            }
            KeyCode::ArrowRight => {
                self.is_turn_right_pressed = is_pressed;
                true
            }
            KeyCode::ArrowUp => {
                self.is_turn_up_pressed = is_pressed;
                true
            }
            KeyCode::ArrowDown => {
                self.is_turn_down_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    pub(crate) fn update_camera(&mut self, camera: &mut Camera) {
        if self.is_turn_left_pressed {
            camera.yaw -= CAMERA_TURN_SPEED;
        }
        if self.is_turn_right_pressed {
            camera.yaw += CAMERA_TURN_SPEED;
        }
        if self.is_turn_up_pressed {
            camera.pitch += CAMERA_TURN_SPEED;
        }
        if self.is_turn_down_pressed {
            camera.pitch -= CAMERA_TURN_SPEED;
        }

        let (delta_x, delta_y) = self.mouse_delta;
        camera.yaw += delta_x * self.mouse_sensitivity;
        camera.pitch -= delta_y * self.mouse_sensitivity;
        self.mouse_delta = (0.0, 0.0);

        camera.pitch = camera.pitch.clamp(-CAMERA_MAX_PITCH, CAMERA_MAX_PITCH);

        let forward = camera.forward();
        let right = camera.right();

        let move_speed = if self.is_sprint_pressed {
            CAMERA_MOVE_SPEED * CAMERA_MOVE_SPEED_SHIFT_MULTIPLIER
        } else {
            CAMERA_MOVE_SPEED
        };

        if self.is_forward_pressed {
            camera.position += forward * move_speed;
        }
        if self.is_backward_pressed {
            camera.position -= forward * move_speed;
        }
        if self.is_right_pressed {
            camera.position += right * move_speed;
        }
        if self.is_left_pressed {
            camera.position -= right * move_speed;
        }
        if self.is_up_pressed {
            camera.position += Vec3::Y * move_speed;
        }
        if self.is_down_pressed {
            camera.position -= Vec3::Y * move_speed;
        }

        camera.update_view_projection();
    }
}
