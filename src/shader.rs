extern crate gl;

use std::str;
use std::ffi::CString;
use std::ptr;
use gl::types::*;

pub type ShaderUnit = GLuint;

pub fn compile (src: &str, ty: GLenum) -> ShaderUnit {
    let typestr = match ty {
        gl::FRAGMENT_SHADER => "Fragment",
        gl::GEOMETRY_SHADER => "Geometry",
        gl::VERTEX_SHADER => "Vertex",
        _ => panic!("Unknown shader type {}", ty)
    };
    unsafe {
        let shader = gl::CreateShader(ty);
        // Attempt to compile the shader
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        let mut len = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
        if len > 0 {
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(
                shader,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
                );
            println!(
                "{} shader info:\n{}",
                typestr,
                str::from_utf8(&buf)
                .ok()
                .expect("ShaderInfoLog not valid utf8")
                );
        }

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            panic!("Failed to compile shader");
        }
        shader
    }
}

pub type Shader = GLuint;

pub fn link (units: &[ShaderUnit]) -> Shader {
    unsafe {
        let program = gl::CreateProgram();
        for unit in units.iter() {
            gl::AttachShader(program, *unit);
        }
        gl::LinkProgram(program);

        let mut len = 0;
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
        if len > 0 {
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetProgramInfoLog(
                program,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
                );
            println!(
                "shader program info:\n{}",
                str::from_utf8(&buf)
                .ok()
                .expect("ProgramInfoLog not valid utf8")
                );
        }

        // Get the link status
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            panic!("Failed to link shader",);
        }
        program
    }
}
