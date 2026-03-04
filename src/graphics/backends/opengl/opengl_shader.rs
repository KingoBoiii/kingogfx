use gl;

pub(crate) struct OpenGLShader {
    program_id: u32,
}

impl OpenGLShader {
    pub(crate) fn from_source(vertex_source: &str, fragment_source: &str) -> Result<Self, String> {
        unsafe {
            let vs = compile_shader(gl::VERTEX_SHADER, vertex_source)?;
            let fs = compile_shader(gl::FRAGMENT_SHADER, fragment_source)?;

            let program = gl::CreateProgram();
            gl::AttachShader(program, vs);
            gl::AttachShader(program, fs);
            gl::LinkProgram(program);

            gl::DeleteShader(vs);
            gl::DeleteShader(fs);

            let mut success = gl::FALSE as i32;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success != (gl::TRUE as i32) {
                let mut len = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = vec![0u8; len.max(1) as usize];
                gl::GetProgramInfoLog(
                    program,
                    len,
                    std::ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut i8,
                );
                gl::DeleteProgram(program);
                let msg = String::from_utf8_lossy(&buffer)
                    .trim_end_matches('\0')
                    .to_string();
                return Err(format!("program link failed: {msg}"));
            }

            Ok(Self { program_id: program })
        }
    }

    pub(crate) fn program_id(&self) -> u32 {
        self.program_id
    }
}

impl Drop for OpenGLShader {
    fn drop(&mut self) {
        if self.program_id != 0 {
            unsafe {
                gl::DeleteProgram(self.program_id);
            }
            self.program_id = 0;
        }
    }
}

fn compile_shader(shader_type: u32, source: &str) -> Result<u32, String> {
    use std::ffi::CString;

    unsafe {
        let shader = gl::CreateShader(shader_type);
        let c_source = CString::new(source).map_err(|_| "shader source contains interior NUL".to_string())?;
        gl::ShaderSource(shader, 1, &c_source.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);

        let mut success = gl::FALSE as i32;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success != (gl::TRUE as i32) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buffer = vec![0u8; len.max(1) as usize];
            gl::GetShaderInfoLog(
                shader,
                len,
                std::ptr::null_mut(),
                buffer.as_mut_ptr() as *mut i8,
            );
            gl::DeleteShader(shader);
            let msg = String::from_utf8_lossy(&buffer).trim_end_matches('\0').to_string();
            return Err(format!("shader compile failed: {msg}"));
        }

        Ok(shader)
    }
}
