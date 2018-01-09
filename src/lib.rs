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

extern crate opengex;
extern crate piston_meta;
extern crate piston_meta_search;
extern crate range;


pub mod graphics;
pub mod drawcalls;
pub mod rawgl;
pub mod rawvk;
pub mod font;
mod shaders;
mod window;
mod vulkano_win_updated;
pub mod settings;
mod model_data;



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
