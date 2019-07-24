#![macro_use]

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
        _ => "Not a valid GLerror",
    }
}

fn get_error() -> Option<&'static str> {
    let error = unsafe { gl::GetError() };
    if error == gl::NO_ERROR {
        None
    } else {
        Some(error_string(error))
    }
}

pub fn print_gl_errors() -> bool {
    if let Some(error) = get_error() {
        eprintln!("GL error: {}", error);
        while let Some(error) = get_error() {
            eprintln!("GL error: {}", error);
        }
        true
    } else {
        false
    }
}

macro_rules! assert_no_gl_error {
    () => {
        if glerror::print_gl_errors() {
            panic!("expected no GL errors")
        }
    };
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
#[repr(u32)]
pub enum GlDebugSeverity {
    High = gl::DEBUG_SEVERITY_HIGH,
    Medium = gl::DEBUG_SEVERITY_MEDIUM,
    Low = gl::DEBUG_SEVERITY_LOW,
    Notification = gl::DEBUG_SEVERITY_NOTIFICATION,
}

#[derive(Debug)]
struct UserParam {
    minimum_severity: GlDebugSeverity,
}

use gl::types::*;
extern "system" fn message_callback(
    source: GLenum,
    gltype: GLenum,
    id: GLuint,
    severity: GLenum,
    length: GLsizei,
    message: *const GLchar,
    user_param: *mut GLvoid,
) {
    let user_param = user_param as *const UserParam;
    let user_param = unsafe { &*user_param };

    if user_param.minimum_severity as GLenum > severity {
        return;
    }

    let fallback;
    let source = match source {
        gl::DEBUG_SOURCE_API => "GL_DEBUG_SOURCE_API",
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => "GL_DEBUG_SOURCE_WINDOW_SYSTEM",
        gl::DEBUG_SOURCE_SHADER_COMPILER => "GL_DEBUG_SOURCE_SHADER_COMPILER",
        gl::DEBUG_SOURCE_APPLICATION => "GL_DEBUG_SOURCE_APPLICATION",
        gl::DEBUG_SOURCE_THIRD_PARTY => "GL_DEBUG_SOURCE_THIRD_PARTY",
        gl::DEBUG_SOURCE_OTHER => "GL_DEBUG_SOURCE_OTHER",
        _ => {
            fallback = format!("Invalid message source {}", source);
            &fallback
        }
    };
    let fallback;
    let gltype = match gltype {
        gl::DEBUG_TYPE_ERROR => "GL_DEBUG_TYPE_ERROR",
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => {
            "GL_DEBUG_TYPE_DEPRECATED_BEHAVIOR"
        }
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "GL_DEBUG_TYPE_UNDEFINED_BEHAVIOR",
        gl::DEBUG_TYPE_PORTABILITY => "GL_DEBUG_TYPE_PORTABILITY",
        gl::DEBUG_TYPE_PERFORMANCE => "GL_DEBUG_TYPE_PERFORMANCE",
        gl::DEBUG_TYPE_MARKER => "GL_DEBUG_TYPE_MARKER",
        gl::DEBUG_TYPE_PUSH_GROUP => "GL_DEBUG_TYPE_PUSH_GROUP",
        gl::DEBUG_TYPE_POP_GROUP => "GL_DEBUG_TYPE_POP_GROUP",
        gl::DEBUG_TYPE_OTHER => "GL_DEBUG_TYPE_OTHER",
        _ => {
            fallback = format!("Invalid message type {}", gltype);
            &fallback
        }
    };
    let fallback;
    let severity = match severity {
        gl::DEBUG_SEVERITY_HIGH => "GL_DEBUG_SEVERITY_HIGH",
        gl::DEBUG_SEVERITY_MEDIUM => "GL_DEBUG_SEVERITY_MEDIUM",
        gl::DEBUG_SEVERITY_LOW => "GL_DEBUG_SEVERITY_LOW",
        gl::DEBUG_SEVERITY_NOTIFICATION => "GL_DEBUG_SEVERITY_NOTIFICATION",
        _ => {
            fallback = format!("Invalid message severity {}", gltype);
            &fallback
        }
    };
    use std::ffi::*;
    let owned_string;
    let message = unsafe {
        if length < 0 {
            CStr::from_ptr(message)
        } else {
            let slice = std::slice::from_raw_parts(
                message as *const u8,
                length as usize,
            );
            owned_string = std::ffi::CString::new(slice).unwrap();
            &owned_string
        }
    };
    dbg!(source);
    dbg!(gltype);
    dbg!(id);
    dbg!(severity);
    dbg!(message);
}

pub fn debug_messages(minimum_severity: GlDebugSeverity) {
    unsafe {
        let user_param = Box::leak(Box::new(UserParam { minimum_severity }));
        gl::DebugMessageCallback(
            message_callback,
            user_param as *const _ as *const _,
        );
    }
}
