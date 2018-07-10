use gl;
use gl::types::*;

pub fn check_glerror() {
  unsafe {
    match gl::GetError() {
      gl::NO_ERROR => {
        println!("GL_ERROR: No Error");
      },
      gl::INVALID_ENUM => {
        println!("GL_ERROR: Invalid enum");
      },
      gl::INVALID_VALUE => {
        println!("GL_ERROR: Invalid value");
      },
      gl::INVALID_OPERATION => {
        println!("GL_ERROR: Invalid operation");
      },
      gl::INVALID_FRAMEBUFFER_OPERATION => {
        println!("GL_ERROR: Invalid framebuffer operation");
      },
      gl::OUT_OF_MEMORY => {
        println!("GL_ERROR: Out of memory");
      },
      gl::STACK_UNDERFLOW => {
        println!("GL_ERROR: Stack Underflow");
      },
      gl::STACK_OVERFLOW => {
        println!("GL_ERROR: Stack Overflow");
      },
      _ => {
        println!("GL_ERROR: Unkown Error");
      }
    }
  }
}

pub fn check_framebufferstatus(fbo: GLuint) {
  unsafe {
    gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
    
    match gl::CheckFramebufferStatus(gl::FRAMEBUFFER) {
      gl::FRAMEBUFFER_COMPLETE => {
        println!("Framebuffer complete!");
      },
      gl::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => {
        println!("Framebuffer incomplete attachment!");
      },
      gl::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => {
        println!("Framebuffer incomplete missing attachment, no image attached to fbo!");
      },
      gl::FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER => {
        println!("Framebuffer incomplete draw buffer!");
      },
      gl::FRAMEBUFFER_INCOMPLETE_READ_BUFFER => {
        println!("Framebuffer incomplete read buffer!");
      },
      gl::FRAMEBUFFER_INCOMPLETE_MULTISAMPLE => {
        println!("Framebuffer incomplete multisample!");
      },
      gl::FRAMEBUFFER_UNSUPPORTED => {
        println!("Framebuffer unsupported!");
      },
      _ => {
        println!("Framebuffer unknown error");
      }
    }
    gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
  }
}
