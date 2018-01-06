extern crate glutin;
extern crate gl;
extern crate cgmath;
extern crate image;

#[macro_use]
extern crate vulkano;

#[macro_use]
extern crate vulkano_shader_derive;

extern crate winit;
extern crate vulkano_win;

pub mod graphics;
pub mod drawcalls;
mod rawgl;
mod rawvk;
pub mod font;
mod shaders;
mod window;
mod vulkano_win_updated;
mod settings;



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
