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
    let mut any_error = false;
    while let Some(error) = get_error() {
        any_error = true;
        eprintln!("GL error: {}", error);
    }
    any_error
}

#[track_caller]
pub fn assert_no_gl_error() {
    if print_gl_errors() {
        panic!("unexpected OpenGL errors")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
#[repr(u32)]
pub enum GlDebugSeverity {
    High = gl::DEBUG_SEVERITY_HIGH,
    Medium = gl::DEBUG_SEVERITY_MEDIUM,
    Low = gl::DEBUG_SEVERITY_LOW,
    Notification = gl::DEBUG_SEVERITY_NOTIFICATION,
}

use std::convert::TryInto;
impl std::convert::TryFrom<GLenum> for GlDebugSeverity {
    type Error = ();

    fn try_from(value: GLenum) -> Result<Self, Self::Error> {
        match value {
            gl::DEBUG_SEVERITY_HIGH => Ok(Self::High),
            gl::DEBUG_SEVERITY_MEDIUM => Ok(Self::Medium),
            gl::DEBUG_SEVERITY_LOW => Ok(Self::Low),
            gl::DEBUG_SEVERITY_NOTIFICATION => Ok(Self::Notification),
            _ => Err(())
        }
    }
}

impl Ord for GlDebugSeverity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let numeric = |&severity| match severity {
            Self::High => 3,
            Self::Medium => 2,
            Self::Low => 1,
            Self::Notification => 0,
        };

        numeric(self).cmp(&numeric(other))
    }
}

impl PartialOrd for GlDebugSeverity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Copy)]
struct UserParam {
    minimum_severity: GlDebugSeverity,
}

use gl::types::*;
#[allow(unused)]
extern "system" fn message_callback(
    source: GLenum,
    gltype: GLenum,
    _id: GLuint,
    severity: GLenum,
    length: GLsizei,
    message: *const GLchar,
    user_param: *mut GLvoid,
) {
    let UserParam { minimum_severity } = unsafe { *(user_param as *const _) };

    let severity = match severity.try_into() {
        Ok(severity) if severity >= minimum_severity => severity,
        _ => {
            return;
        }
    };

    let fallback;
    let source = match source {
        gl::DEBUG_SOURCE_API => "API",
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => "window system",
        gl::DEBUG_SOURCE_SHADER_COMPILER => "shader compiler",
        gl::DEBUG_SOURCE_APPLICATION => "application",
        gl::DEBUG_SOURCE_THIRD_PARTY => "third party",
        gl::DEBUG_SOURCE_OTHER => "unspecified source",
        _ => {
            fallback = format!("Unknown message source {}", source);
            &fallback
        }
    };

    let fallback;
    let gltype = match gltype {
        gl::DEBUG_TYPE_ERROR => "error",
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "deprecated behavior",
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "undefined behavior",
        gl::DEBUG_TYPE_PORTABILITY => "portability",
        gl::DEBUG_TYPE_PERFORMANCE => "performance",
        gl::DEBUG_TYPE_MARKER => "marker",
        gl::DEBUG_TYPE_PUSH_GROUP => "push debug group",
        gl::DEBUG_TYPE_POP_GROUP => "pop debug group",
        gl::DEBUG_TYPE_OTHER => "GL_DEBUG_TYPE_OTHER",
        _ => {
            fallback = format!("{{Unknown message type {}}}", gltype);
            &fallback
        }
    };

    let severity = match severity {
        GlDebugSeverity::High => "high",
        GlDebugSeverity::Medium => "medium",
        GlDebugSeverity::Low => "low",
        GlDebugSeverity::Notification => "notification",
    };

    use std::ffi::*;
    let message = unsafe {
        if length < 0 {
            CStr::from_ptr(message)
        } else {
            let slice = std::slice::from_raw_parts(
                message as *const u8,
                length as usize + 1,
            );
            CStr::from_bytes_with_nul(slice).unwrap()
        }
    };

    eprintln!(
        "{} severity debug message about {} from {}: {}",
        severity,
        gltype,
        source,
        message.to_string_lossy(),
    );
}

#[allow(unused)]
pub fn debug_messages(minimum_severity: GlDebugSeverity) {
    unsafe {
        let user_param = Box::leak(Box::new(UserParam { minimum_severity }));
        gl::DebugMessageCallback(
            Some(message_callback),
            user_param as *const _ as *const _,
        );
    }
}
