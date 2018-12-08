fn error_string(error: gl::types::GLuint) -> &'static str {
    match error {
        gl::NO_ERROR => "GL_NO_ERROR",
        gl::INVALID_ENUM => "GL_INVALID_ENUM",
        gl::INVALID_VALUE => "GL_INVALID_VALUE",
        gl::INVALID_OPERATION => "GL_INVALID_OPERATION",
        gl::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
        gl::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
        gl::STACK_UNDERFLOW => "GL_STACK_UNDERFLOW",
        gl::STACK_OVERFLOW => "GL_STACK_OVERFLOW",
        _ => "Not a valid GLerror"
    }
}

fn get_error() -> Option<&'static str> {
    let error = unsafe { gl::GetError() };
    if error == gl::NO_ERROR { None }
    else { Some(error_string(error)) }
}

pub fn print_gl_errors() -> bool {
    if let Some(error) = get_error() {
        eprintln!("GL error: {}", error);
        while let Some(error) = get_error() {
            eprintln!("GL error: {}", error);
        }
        true
    } else { false }
}

#[macro_export]
macro_rules! assert_no_gl_error {
    () => {
        if glerror::print_gl_errors() { panic!("expected no GL errors") }
    }
}
