//! The viewer windows: a watched GDS file or a probe channel → frames on
//! screen via [`crate::gpu`]; pan, zoom and refit on input.
//!
//! Contents:
//! - `pub struct Probe`
//! - `pub fn run_viewer(path: PathBuf, layer_names: LayerMap)`
//! - `Probe::open(title: &str) -> Self`
//! - `Probe::wait(&mut self)`
//! - `Probe::send(&self, polys: &[Poly], title: Option<&str>)`
//! - `pub fn show_macro(mac: &analog::base::Macro, title: &str, gds: &[(u16, u16)])`

use std::{
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant, SystemTime},
};

use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

use crate::gds::{parse_gds, polys_from_shapes, Poly, TextEntry};
use crate::gpu::Gpu;
use crate::mesh::{build_legend, fit_view, outline_vertices, text_vertices, triangulate, CamUni};
use crate::style::LayerMap;

/// A window on a background thread showing the latest polygons sent to it.
/// Stays open after the sender is dropped; a no-op without a display.
pub struct Probe {
    tx: Option<mpsc::Sender<ProbeMsg>>,
    handle: Option<std::thread::JoinHandle<()>>,
}

enum Source {
    File { path: PathBuf, mtime: Option<SystemTime>, last_poll: Instant },
    Channel(mpsc::Receiver<ProbeMsg>),
}

struct ProbeMsg {
    polys: Vec<Poly>,
    title: Option<String>,
}

struct App {
    win: Option<std::sync::Arc<Window>>,
    gpu: Option<Gpu>,
    cam: CamUni,
    cursor: [f64; 2],
    drag: bool,
    fit: bool,
    title: String,
    source: Source,
    layer_names: LayerMap,
    /// The last geometry shown, re-uploaded once the window exists.
    shown: (Vec<Poly>, Vec<TextEntry>),
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.win.is_some() {
            return;
        }
        let attrs = Window::default_attributes().with_title(&self.title).with_inner_size(LogicalSize::new(1280u32, 720u32));
        let w = std::sync::Arc::new(el.create_window(attrs).unwrap());
        let s = w.inner_size();
        self.cam.aspect = s.width as f32 / s.height.max(1) as f32;
        self.gpu = Some(Gpu::new(w.clone()));
        self.win = Some(w);
        if matches!(self.source, Source::File { .. }) {
            self.load_gds();
        } else {
            let (polys, texts) = std::mem::take(&mut self.shown);
            self.show(polys, texts);
        }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _: WindowId, ev: WindowEvent) {
        match ev {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(s) => {
                self.cam.aspect = s.width as f32 / s.height.max(1) as f32;
                if let Some(g) = &mut self.gpu {
                    g.resize(s.width, s.height);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(g) = &mut self.gpu {
                    g.draw(&self.cam);
                }
            }
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                self.drag = state == ElementState::Pressed;
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (dx, dy) = (position.x - self.cursor[0], position.y - self.cursor[1]);
                self.cursor = [position.x, position.y];
                if let (true, Some(w)) = (self.drag, &self.win) {
                    let s = w.inner_size();
                    self.cam.offset[0] += (2.0 * dx as f32 / s.width as f32) * self.cam.aspect / self.cam.scale;
                    self.cam.offset[1] -= (2.0 * dy as f32 / s.height as f32) / self.cam.scale;
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let d = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 / 100.0,
                };
                // Zoom about the cursor.
                let ndc = self.cursor_ndc();
                let inv_old = 1.0 / self.cam.scale;
                self.cam.scale *= if d > 0.0 { 1.1 } else { 1.0 / 1.1 };
                let diff = 1.0 / self.cam.scale - inv_old;
                self.cam.offset[0] += ndc[0] * self.cam.aspect * diff;
                self.cam.offset[1] += ndc[1] * diff;
            }
            WindowEvent::KeyboardInput { event: ev, .. } if ev.state == ElementState::Pressed => {
                let step = 0.1 / self.cam.scale;
                match ev.logical_key {
                    Key::Named(NamedKey::Escape) => el.exit(),
                    Key::Named(NamedKey::ArrowUp) => self.cam.offset[1] += step,
                    Key::Named(NamedKey::ArrowDown) => self.cam.offset[1] -= step,
                    Key::Named(NamedKey::ArrowLeft) => self.cam.offset[0] -= step,
                    Key::Named(NamedKey::ArrowRight) => self.cam.offset[0] += step,
                    Key::Character(ref c) => match c.as_str() {
                        "r" => {
                            self.fit = true;
                            self.load_gds();
                        }
                        "=" | "+" => self.cam.scale *= 1.2,
                        "-" => self.cam.scale /= 1.2,
                        _ => {}
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _el: &ActiveEventLoop) {
        match &mut self.source {
            Source::File { path, mtime, last_poll } => {
                if last_poll.elapsed() >= Duration::from_millis(500) {
                    *last_poll = Instant::now();
                    let mt = std::fs::metadata(&*path).and_then(|m| m.modified()).ok();
                    if mt.is_some() && mt != *mtime {
                        self.load_gds();
                    }
                }
            }
            Source::Channel(rx) => {
                if let Some(msg) = rx.try_iter().last() {
                    if let (Some(t), Some(win)) = (&msg.title, &self.win) {
                        win.set_title(&format!("{} \u{2014} {t}", self.title));
                    }
                    self.show(msg.polys, Vec::new());
                }
            }
        }
        if let Some(w) = &self.win {
            w.request_redraw();
        }
    }
}

/// Open a window on a GDS file, reloading it whenever it changes on disk.
pub fn run_viewer(path: PathBuf, layer_names: LayerMap) {
    let el = EventLoop::new().unwrap_or_else(|e| {
        eprintln!("cannot create window (no display server?): {e}");
        std::process::exit(1);
    });
    let title = format!("GDS \u{2014} {}", path.display());
    let source = Source::File { path, mtime: None, last_poll: Instant::now() };
    el.run_app(&mut App::new(title, source, layer_names)).unwrap();
}

impl Probe {
    /// Spawns the window thread and returns at once; without a display the
    /// thread prints why and exits, and every later `send` is dropped.
    #[must_use]
    pub fn open(title: &str) -> Self {
        let (tx, rx) = mpsc::channel();
        let title = title.to_string();
        let handle = std::thread::spawn(move || {
            let el = match make_event_loop() {
                Ok(el) => el,
                Err(e) => {
                    eprintln!("probe: no display: {e}");
                    return;
                }
            };
            el.run_app(&mut App::new(title, Source::Channel(rx), LayerMap::new())).unwrap();
        });
        Probe { tx: Some(tx), handle: Some(handle) }
    }

    /// Stop sending and block until the user closes the window.
    pub fn wait(&mut self) {
        self.tx.take();
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }

    /// Copies `polys` to the window, which shows only the latest message;
    /// a no-op after [`Probe::wait`] or once the window closed.
    pub fn send(&self, polys: &[Poly], title: Option<&str>) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(ProbeMsg { polys: polys.to_vec(), title: title.map(String::from) });
        }
    }
}

/// Show a macro's geometry (GDS table as in [`polys_from_shapes`]) and block
/// until the window closes.
pub fn show_macro(mac: &analog::base::Macro, title: &str, gds: &[(u16, u16)]) {
    let mut probe = Probe::open(title);
    probe.send(&polys_from_shapes(&mac.shapes, gds), Some(title));
    probe.wait();
}

// ---- private ----

impl App {
    fn new(title: String, source: Source, layer_names: LayerMap) -> Self {
        Self {
            win: None,
            gpu: None,
            cam: CamUni { offset: [0.0; 2], scale: 1.0, aspect: 1.0 },
            cursor: [0.0; 2],
            drag: false,
            fit: true,
            title,
            source,
            layer_names,
            shown: (Vec::new(), Vec::new()),
        }
    }

    fn show(&mut self, mut polys: Vec<Poly>, texts: Vec<TextEntry>) {
        polys.sort_by_key(Poly::key);
        if self.fit && !polys.is_empty() {
            self.cam = fit_view(&polys, self.cam.aspect);
            self.fit = false;
        }
        if let Some(gpu) = &mut self.gpu {
            let mut fill = triangulate(&polys, &self.layer_names);
            fill.extend(text_vertices(&texts, &polys));
            gpu.fill = gpu.mesh(&fill);
            gpu.line = gpu.mesh(&outline_vertices(&polys, &self.layer_names));
            gpu.overlay = gpu.mesh(&build_legend(&polys, &self.layer_names));
        }
        self.shown = (polys, texts);
    }

    fn load_gds(&mut self) {
        let Source::File { path, mtime, .. } = &mut self.source else { return };
        let data = match std::fs::read(&*path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("read {}: {e}", path.display());
                return;
            }
        };
        *mtime = std::fs::metadata(&*path).and_then(|m| m.modified()).ok();
        let (polys, texts) = parse_gds(&data);
        self.show(polys, texts);
    }

    fn cursor_ndc(&self) -> [f32; 2] {
        self.win.as_ref().map_or([0.0; 2], |w| {
            let s = w.inner_size();
            [2.0 * self.cursor[0] as f32 / s.width as f32 - 1.0, -(2.0 * self.cursor[1] as f32 / s.height as f32 - 1.0)]
        })
    }
}

/// An event loop that runs off the main thread (the probe's window thread).
#[cfg(target_os = "linux")]
fn make_event_loop() -> Result<EventLoop<()>, winit::error::EventLoopError> {
    let mut builder = EventLoop::builder();
    winit::platform::x11::EventLoopBuilderExtX11::with_any_thread(&mut builder, true);
    winit::platform::wayland::EventLoopBuilderExtWayland::with_any_thread(&mut builder, true);
    builder.build()
}

#[cfg(not(target_os = "linux"))]
fn make_event_loop() -> Result<EventLoop<()>, winit::error::EventLoopError> {
    EventLoop::new()
}
