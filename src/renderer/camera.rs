use glam::f32::Vec3;
use wgpu::util::DeviceExt as _;
use winit::keyboard::KeyCode;

const CAMERA_STARTING_POSITION: Vec3 = Vec3::new(0.0, 1.0, 2.0);
const CAMERA_STARTING_TARGET: Vec3 = Vec3::new(0.0, 0.0, 0.0);
const CAMERA_FOV_Y: f32 = 90.0;
const CAMERA_Z_NEAR: f32 = 0.1;
const CAMERA_Z_FAR: f32 = 100.0;
const CAMERA_SPEED: f32 = 0.03;

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
    target: Vec3,
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
        let target = CAMERA_STARTING_TARGET;
        let aspect_ratio = surface_config.width as f32 / surface_config.height as f32;

        let view = glam::Mat4::look_at_rh(position, target, Vec3::Y);
        let projection =
            glam::Mat4::perspective_rh(CAMERA_FOV_Y, aspect_ratio, CAMERA_Z_NEAR, CAMERA_Z_FAR);
        let view_projection = projection * view;

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
            target,
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

    fn update_view_projection(&mut self) {
        let view = glam::Mat4::look_at_rh(self.position, self.target, Vec3::Y);
        let proj = glam::Mat4::perspective_rh(
            CAMERA_FOV_Y,
            self.aspect_ratio,
            CAMERA_Z_NEAR,
            CAMERA_Z_FAR,
        );
        self.uniform.view_projection = (proj * view).to_cols_array_2d();
    }

    pub(crate) fn update_buffer(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }
}

pub(crate) struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    pub(crate) fn new() -> Self {
        let speed = CAMERA_SPEED;
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub(crate) fn handle_key(&mut self, code: KeyCode, is_pressed: bool) -> bool {
        match code {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.is_forward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.is_backward_pressed = is_pressed;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    pub(crate) fn update_camera(&self, camera: &mut Camera) {
        let forward_vector = camera.target - camera.position;
        let forward_normalized = forward_vector.normalize();
        let forward_magnitude = forward_vector.length();

        // Prevents glitching when the camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_magnitude > self.speed {
            camera.position += forward_normalized * self.speed;
        }
        if self.is_backward_pressed {
            camera.position -= forward_normalized * self.speed;
        }

        let right = forward_normalized.cross(Vec3::Y);

        // Redo radius calc in case the forward/backward is pressed.
        let forward_vector = camera.target - camera.position;
        let forward_magnitude = forward_vector.length();

        if self.is_right_pressed {
            // Rescale the distance between the target and the eye so
            // that it doesn't change. The eye, therefore, still
            // lies on the circle made by the target and eye.
            camera.position = camera.target
                - (forward_vector + right * self.speed).normalize() * forward_magnitude;
        }
        if self.is_left_pressed {
            camera.position = camera.target
                - (forward_vector - right * self.speed).normalize() * forward_magnitude;
        }

        camera.update_view_projection();
    }
}
