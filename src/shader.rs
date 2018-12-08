use std::str;
use std::ffi::CString;
use std::ptr;
use gl::types::*;
use glerror::*;

type ShaderUnit = GLuint;

fn compile (src: &str, ty: GLenum) -> ShaderUnit {
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

#[derive(Debug)]
pub struct Shader {
    pub name: GLuint,
    // locations: Vec<GLint>,
}

impl Shader {
    pub fn from_sources (sources: &[(&str, GLenum)]) -> Shader {
        let mut units = Vec::new();
        for (source, ty) in sources {
            units.push(compile(source, *ty));
        }
        link(&units)
    }

    #[allow(dead_code)]
    pub fn get_location (&self, name: &str) -> GLint {
        use std::ffi::CString;
        let location = {
            let c_name = CString::new(name).expect("uniform name is not a valid c string");
            unsafe { gl::GetUniformLocation(self.name, c_name.as_ptr() as * const GLchar) }
        };

        if location == -1 {
            print_gl_errors();
            panic!("Could not get location of uniform '{}' in program {}",
                   name,
                   self.name,
                   );
        }
        location
    }

    pub fn activate (&self) {
        unsafe { gl::UseProgram(self.name) };
        print_gl_errors();
    }
}

fn link (units: &[ShaderUnit]) -> Shader {
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
        Shader { name: program }
    }
}
