//! Vertex lists → pixels: the wgpu device, surface and the three pipelines
//! (fill, outline, screen-space overlay) with their WGSL shaders.
//!
//! Contents:
//! - `pub(crate) const SHADER`
//! - `pub(crate) const OVERLAY_SHADER`
//! - `pub(crate) struct Mesh`
//! - `pub(crate) struct Gpu`
//! - `Gpu::new(win: std::sync::Arc<Window>) -> Self`
//! - `Gpu::resize(&mut self, w: u32, h: u32)`
//! - `Gpu::mesh(&self, verts: &[Vertex]) -> Mesh`
//! - `Gpu::draw(&mut self, cam: &CamUni)`

use winit::window::Window;

use crate::mesh::{CamUni, Vertex};

pub(crate) const SHADER: &str = include_str!("layout.wgsl");

/// Screen-space: positions are already NDC.
pub(crate) const OVERLAY_SHADER: &str = include_str!("overlay.wgsl");

/// A vertex buffer and its vertex count.
pub(crate) struct Mesh {
    buf: Option<wgpu::Buffer>,
    n: u32,
}

pub(crate) struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    fill_pipeline: wgpu::RenderPipeline,
    line_pipeline: wgpu::RenderPipeline,
    overlay_pipeline: wgpu::RenderPipeline,
    cam_buf: wgpu::Buffer,
    cam_bg: wgpu::BindGroup,
    pub(crate) fill: Mesh,
    pub(crate) line: Mesh,
    pub(crate) overlay: Mesh,
}

impl Gpu {
    pub(crate) fn new(win: std::sync::Arc<Window>) -> Self {
        let sz = win.inner_size();
        let inst = wgpu::Instance::default();
        let surface = inst.create_surface(win).unwrap();
        let adapter =
            pollster::block_on(inst.request_adapter(&wgpu::RequestAdapterOptions { compatible_surface: Some(&surface), ..Default::default() }))
                .expect("no GPU adapter found");
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None)).unwrap();

        let caps = surface.get_capabilities(&adapter);
        let fmt = caps.formats[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: fmt,
            width: sz.width.max(1),
            height: sz.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let module =
            |src: &str| device.create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source: wgpu::ShaderSource::Wgsl(src.into()) });
        let (shader, overlay_shader) = (module(SHADER), module(OVERLAY_SHADER));
        let cam_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<CamUni>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            }],
        });
        let cam_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: cam_buf.as_entire_binding() }],
        });
        let layout = |bgls: &[&wgpu::BindGroupLayout]| {
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: None, bind_group_layouts: bgls, push_constant_ranges: &[] })
        };
        let (pll, overlay_pll) = (layout(&[&bgl]), layout(&[]));
        use wgpu::PrimitiveTopology::{LineList, TriangleList};
        Self {
            fill_pipeline: make_pipeline(&device, &shader, &pll, fmt, TriangleList),
            line_pipeline: make_pipeline(&device, &shader, &pll, fmt, LineList),
            overlay_pipeline: make_pipeline(&device, &overlay_shader, &overlay_pll, fmt, TriangleList),
            device,
            queue,
            surface,
            config,
            cam_buf,
            cam_bg,
            fill: Mesh { buf: None, n: 0 },
            line: Mesh { buf: None, n: 0 },
            overlay: Mesh { buf: None, n: 0 },
        }
    }

    pub(crate) fn resize(&mut self, w: u32, h: u32) {
        if w > 0 && h > 0 {
            self.config.width = w;
            self.config.height = h;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub(crate) fn mesh(&self, verts: &[Vertex]) -> Mesh {
        if verts.is_empty() {
            return Mesh { buf: None, n: 0 };
        }
        let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of_val(verts) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&buf, 0, bytemuck::cast_slice(verts));
        Mesh { buf: Some(buf), n: verts.len() as u32 }
    }

    pub(crate) fn draw(&mut self, cam: &CamUni) {
        self.queue.write_buffer(&self.cam_buf, 0, bytemuck::bytes_of(cam));
        let Ok(frame) = self.surface.get_current_texture() else {
            self.surface.configure(&self.device, &self.config);
            return;
        };
        let view = frame.texture.create_view(&Default::default());
        let mut enc = self.device.create_command_encoder(&Default::default());
        {
            let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.08, g: 0.08, b: 0.10, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            for (mesh, pipeline, world) in
                [(&self.fill, &self.fill_pipeline, true), (&self.line, &self.line_pipeline, true), (&self.overlay, &self.overlay_pipeline, false)]
            {
                let Some(buf) = &mesh.buf else { continue };
                rp.set_pipeline(pipeline);
                if world {
                    rp.set_bind_group(0, &self.cam_bg, &[]);
                }
                rp.set_vertex_buffer(0, buf.slice(..));
                rp.draw(0..mesh.n, 0..1);
            }
        }
        self.queue.submit([enc.finish()]);
        frame.present();
    }
}

// ---- private ----

fn make_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    layout: &wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
    topology: wgpu::PrimitiveTopology,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs"),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4],
            }],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs"),
            targets: &[Some(wgpu::ColorTargetState { format, blend: Some(wgpu::BlendState::ALPHA_BLENDING), write_mask: wgpu::ColorWrites::ALL })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState { topology, ..Default::default() },
        depth_stencil: None,
        multisample: Default::default(),
        multiview: None,
        cache: None,
    })
}
