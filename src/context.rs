use gl;
use luminance::context::GraphicsContext;
use luminance::state::GraphicsState;
use sdl2;
use std::cell::RefCell;
use std::rc::Rc;

pub struct SdlContext {
    pub sdl: sdl2::Sdl,
    pub window: sdl2::video::Window,
    _gl_context: sdl2::video::GLContext,
    state: Rc<RefCell<GraphicsState>>,
}

impl SdlContext {
    pub fn new(width: u32, height: u32) -> Self {
        let sdl = sdl2::init().expect("Could not init sdl2");

        let video_system =
            sdl.video().expect("Could not initialize video system");
        sdl.mouse().set_relative_mouse_mode(true);

        let gl_attr = video_system.gl_attr();

        gl_attr.set_context_major_version(3);
        gl_attr.set_context_minor_version(3);
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_flags().forward_compatible().set();

        let window = video_system
            .window("Hello", width, height)
            .opengl()
            .fullscreen_desktop()
            .build()
            .expect("Could not create window");

        let _gl_context = window
            .gl_create_context()
            .expect("Could not create OpenGL context");

        gl::load_with(|s| video_system.gl_get_proc_address(s) as *const _);

        let state = GraphicsState::new()
            .expect("Only one graphics state per thread allowed");

        Self {
            sdl,
            window,
            _gl_context,
            state: Rc::new(RefCell::new(state)),
        }
    }
}

unsafe impl GraphicsContext for SdlContext {
    fn state(&self) -> &Rc<RefCell<GraphicsState>> {
        &self.state
    }
}
