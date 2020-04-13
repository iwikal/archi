use luminance::context::GraphicsContext;
use luminance_sdl2::GL33Surface;

pub struct Context {
    pub surface: GL33Surface,
}

impl Context {
    pub fn new() -> Self {
        let surface = GL33Surface::build_with(|video| {
            let mut builder = video.window("archi", 1080, 720);
            builder.fullscreen_desktop();
            builder
        })
        .unwrap();

        let mouse = surface.sdl().mouse();
        mouse.set_relative_mouse_mode(true);

        Self { surface }
    }
}

unsafe impl GraphicsContext for Context {
    type Backend = <GL33Surface as GraphicsContext>::Backend;

    fn backend(&mut self) -> &mut Self::Backend {
        self.surface.backend()
    }
}
