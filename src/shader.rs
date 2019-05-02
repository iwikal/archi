use gl::types::*;
use glerror::*;
use std::ffi::CString;
use std::ptr;
use std::str;

struct ShaderUnit {
    name: GLuint,
}

impl ShaderUnit {
    fn new(src: &str, ty: GLenum) -> Self {
        let typestr = match ty {
            gl::FRAGMENT_SHADER => "Fragment",
            gl::GEOMETRY_SHADER => "Geometry",
            gl::VERTEX_SHADER => "Vertex",
            _ => panic!("Unknown shader type {}", ty),
        };
        unsafe {
            let name = gl::CreateShader(ty);
            // Attempt to compile the shader
            let c_str = CString::new(src.as_bytes()).unwrap();
            gl::ShaderSource(name, 1, &c_str.as_ptr(), ptr::null());
            gl::CompileShader(name);

            let mut len = 0;
            gl::GetShaderiv(name, gl::INFO_LOG_LENGTH, &mut len);
            if len > 0 {
                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
                gl::GetShaderInfoLog(name, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
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
            gl::GetShaderiv(name, gl::COMPILE_STATUS, &mut status);

            // Fail on error
            if status != (gl::TRUE as GLint) {
                panic!("Failed to compile shader");
            }
            ShaderUnit { name }
        }
    }
}

impl Drop for ShaderUnit {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.name);
        }
    }
}

#[derive(Debug)]
pub struct Shader {
    pub name: GLuint,
    // locations: Vec<GLint>,
}

impl Shader {
    fn link(units: &[ShaderUnit]) -> Self {
        unsafe {
            let program = gl::CreateProgram();
            for unit in units.iter() {
                gl::AttachShader(program, unit.name);
            }
            gl::LinkProgram(program);
            for unit in units.iter() {
                gl::DetachShader(program, unit.name);
            }

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

    pub fn from_sources(sources: &[(&str, GLenum)]) -> Self {
        let mut units = Vec::new();
        for (source, ty) in sources {
            units.push(ShaderUnit::new(source, *ty));
        }
        Self::link(&units)
    }

    pub fn from_vert_frag(vert: &str, frag: &str) -> Self {
        let sources = [(vert, gl::VERTEX_SHADER), (frag, gl::FRAGMENT_SHADER)];
        Self::from_sources(&sources)
    }

    #[allow(dead_code)]
    pub fn get_location(&self, name: &str) -> GLint {
        let location = {
            let c_name = CString::new(name).expect("uniform name is not a valid c string");
            unsafe { gl::GetUniformLocation(self.name, c_name.as_ptr() as *const GLchar) }
        };

        if location == -1 {
            print_gl_errors();
            panic!(
                "Could not get location of uniform '{}' in program {}",
                name, self.name,
            );
        }
        location
    }

    pub fn activate(&self) {
        unsafe {
            gl::UseProgram(self.name);
        }
    }

    pub fn deactivate() {
        unsafe {
            gl::UseProgram(0);
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.name) }
    }
}
