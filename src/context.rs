use luminance::{
    framebuffer::{Framebuffer, FramebufferError},
    texture::Dim2,
};
use luminance_gl::GL33;

pub struct Context {
    pub ctx: glutin::WindowedContext<glutin::PossiblyCurrent>,
    gl: GL33,
}

impl Context {
    pub fn new(event_loop: &glutin::event_loop::EventLoop<()>) -> Self {
        let primary_monitor = event_loop.primary_monitor();

        let window_builder = glutin::window::WindowBuilder::new()
            .with_fullscreen(Some(glutin::window::Fullscreen::Borderless(
                primary_monitor,
            )))
            .with_visible(false);

        let windowed_ctx = glutin::ContextBuilder::new()
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 3)))
            .with_gl_profile(glutin::GlProfile::Core)
            .build_windowed(window_builder, event_loop)
            .unwrap();

        let ctx =
            unsafe { windowed_ctx.make_current().map_err(|(_, e)| e).unwrap() };

        // init OpenGL
        gl::load_with(|s| ctx.get_proc_address(s) as *const std::ffi::c_void);

        ctx.window().set_cursor_visible(false);

        let gl = GL33::new().unwrap();

        Self { ctx, gl }
    }

    /// Get the underlying size (in physical pixels) of the surface.
    ///
    /// This is equivalent to getting the inner size of the windowed context and converting it to
    /// a physical size by using the HiDPI factor of the windowed context.
    pub fn size(&self) -> [u32; 2] {
        let size = self.ctx.window().inner_size();
        [size.width, size.height]
    }

    /// Get access to the back buffer.
    pub fn back_buffer(
        &mut self,
    ) -> Result<Framebuffer<GL33, Dim2, (), ()>, FramebufferError> {
        Framebuffer::back_buffer(self, self.size())
    }

    /// Swap the back and front buffers.
    pub fn swap_buffers(&mut self) {
        let _ = self.ctx.swap_buffers();
    }
}

unsafe impl luminance::context::GraphicsContext for Context {
    type Backend = GL33;

    fn backend(&mut self) -> &mut Self::Backend {
        &mut self.gl
    }
}
