use gl;

pub(crate) type GlShaderBinaryFn = unsafe extern "system" fn(
    count: i32,
    shaders: *const u32,
    binary_format: u32,
    binary: *const std::ffi::c_void,
    length: i32,
);

pub(crate) type GlSpecializeShaderFn = unsafe extern "system" fn(
    shader: u32,
    entry_point: *const i8,
    num_specialization_constants: u32,
    constant_index: *const u32,
    constant_value: *const u32,
);

// GL_ARB_gl_spirv / OpenGL 4.6
pub(crate) const SHADER_BINARY_FORMAT_SPIR_V_ARB: u32 = 0x9551;

pub(crate) struct OpenGLShader {
    program_id: u32,
}

impl OpenGLShader {
    pub(crate) fn from_spirv(
        vertex_spv: &[u32],
        fragment_spv: &[u32],
        shader_binary: GlShaderBinaryFn,
        specialize_shader: GlSpecializeShaderFn,
    ) -> Result<Self, String> {
        unsafe {
            let vs = load_spirv_shader(gl::VERTEX_SHADER, vertex_spv, "main", shader_binary, specialize_shader)?;
            let fs = load_spirv_shader(gl::FRAGMENT_SHADER, fragment_spv, "main", shader_binary, specialize_shader)?;

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

fn load_spirv_shader(
    shader_type: u32,
    spv: &[u32],
    entry_point: &str,
    shader_binary: GlShaderBinaryFn,
    specialize_shader: GlSpecializeShaderFn,
) -> Result<u32, String> {
    use std::ffi::CString;

    unsafe {
        let shader = gl::CreateShader(shader_type);

        // OpenGL expects a byte slice for the SPIR-V binary.
        let spv_bytes: &[u8] = std::slice::from_raw_parts(
            spv.as_ptr() as *const u8,
            spv.len() * std::mem::size_of::<u32>(),
        );

        shader_binary(
            1,
            &shader,
            SHADER_BINARY_FORMAT_SPIR_V_ARB,
            spv_bytes.as_ptr().cast(),
            spv_bytes.len() as i32,
        );

        let entry = CString::new(entry_point).map_err(|_| "entry point contains interior NUL".to_string())?;
        specialize_shader(shader, entry.as_ptr(), 0, std::ptr::null(), std::ptr::null());

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
            return Err(format!("SPIR-V shader specialize failed: {msg}"));
        }

        Ok(shader)
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
