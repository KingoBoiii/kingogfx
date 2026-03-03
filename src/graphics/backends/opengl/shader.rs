use crate::graphics::shader::ShaderBackend;
use gl::types::*;
use std::ffi::{CString};

pub struct OpenGLShader {
  pub program_id: u32,
}

impl OpenGLShader {
  pub fn from_source(vertex_source: &str, fragment_source: &str) -> Result<Self, String> {
    unsafe {
      // Compile vertex shader
      let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
      let c_vertex_source = CString::new(vertex_source).unwrap();
      gl::ShaderSource(vertex_shader, 1, &c_vertex_source.as_ptr(), std::ptr::null());
      gl::CompileShader(vertex_shader);

      // Check vertex shader compile status
      let mut success = gl::FALSE as GLint;
      gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
      if success != gl::TRUE as GLint {
        let mut len = 0;
        gl::GetShaderiv(vertex_shader, gl::INFO_LOG_LENGTH, &mut len);
        let mut buffer = Vec::with_capacity(len as usize);
        buffer.set_len((len as usize) - 1);
        gl::GetShaderInfoLog(vertex_shader, len, std::ptr::null_mut(), buffer.as_mut_ptr() as *mut GLchar);
        let error = String::from_utf8_lossy(&buffer).into_owned();
        return Err(format!("Vertex shader compilation failed: {}", error));
      }

      // Compile fragment shader
      let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
      let c_fragment_source = CString::new(fragment_source).unwrap();
      gl::ShaderSource(fragment_shader, 1, &c_fragment_source.as_ptr(), std::ptr::null());
      gl::CompileShader(fragment_shader);

      // Check fragment shader compile status
      let mut success = gl::FALSE as GLint;
      gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
      if success != gl::TRUE as GLint {
        let mut len = 0;
        gl::GetShaderiv(fragment_shader, gl::INFO_LOG_LENGTH, &mut len);
        let mut buffer = Vec::with_capacity(len as usize);
        buffer.set_len((len as usize) - 1);
        gl::GetShaderInfoLog(fragment_shader, len, std::ptr::null_mut(), buffer.as_mut_ptr() as *mut GLchar);
        let error = String::from_utf8_lossy(&buffer).into_owned();
        return Err(format!("Fragment shader compilation failed: {}", error));
      }

      // Link shaders into a program
      let program_id = gl::CreateProgram();
      gl::AttachShader(program_id, vertex_shader);
      gl::AttachShader(program_id, fragment_shader);
      gl::LinkProgram(program_id);

      // Check link status
      let mut success = gl::FALSE as GLint;
      gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
      if success != gl::TRUE as GLint {
        let mut len = 0;
        gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
        let mut buffer = Vec::with_capacity(len as usize);
        buffer.set_len((len as usize) - 1);
        gl::GetProgramInfoLog(program_id, len, std::ptr::null_mut(), buffer.as_mut_ptr() as *mut GLchar);
        let error = String::from_utf8_lossy(&buffer).into_owned();
        return Err(format!("Shader program linking failed: {}", error));
      }

      // Clean up shaders (no longer needed after linking)
      gl::DeleteShader(vertex_shader);
      gl::DeleteShader(fragment_shader);

      Ok(OpenGLShader { program_id })
    }
  }
}

impl ShaderBackend for OpenGLShader {
  fn bind(&self) {
    unsafe {
      gl::UseProgram(self.program_id);
    }
  }

  fn unbind(&self) {
    unsafe {
      gl::UseProgram(0);
    }
  }
}