use std::str;
use std::ffi::CString;
use std::ptr;
use gl::types::*;

type ShaderUnit = GLuint;

fn gl_errors() -> Vec<&'static str> {
    let mut strings = Vec::new();
    loop {
        let error = unsafe { gl::GetError() };
        if error == gl::NO_ERROR { break; }
        strings.push(match error {
            gl::INVALID_ENUM => "GL_INVALID_ENUM",
            gl::INVALID_VALUE => "GL_INVALID_VALUE",
            gl::INVALID_OPERATION => "GL_INVALID_OPERATION",
            gl::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
            gl::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
            gl::STACK_UNDERFLOW => "GL_STACK_UNDERFLOW",
            gl::STACK_OVERFLOW => "GL_STACK_OVERFLOW",
            _ => "Oh no, glGetError itself failed!"
        });
    }
    strings
}

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

    pub fn get_location (&self, name: &str) -> GLint {
        use std::ffi::CString;
        let location = {
            let c_name = CString::new(name).expect("uniform name is not a valid c string");
            unsafe { gl::GetUniformLocation(self.name, c_name.as_ptr() as * const GLchar) }
        };

        if location == -1 {
            panic!("Could not get location of uniform '{}' in program {}{}",
                   name,
                   self.name,
                   [vec![""], gl_errors()].concat().join("\n"));
        }
        location
    }

    pub fn activate (&self) {
        unsafe { gl::UseProgram(self.name) };
        let errors = gl_errors();
        if errors.len() > 0 { println!("GL error: {}", errors.join("\n")); }
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
