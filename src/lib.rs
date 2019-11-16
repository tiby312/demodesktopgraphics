
mod gl {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}


mod shader;
use shader::*;

//extern crate glutin;
extern crate axgeom;

pub use glutin;
use axgeom::*;
use crate::gl::types::*;
use std::mem;
use std::ptr;
use std::str;
use std::ffi::CString;
use core::marker::PhantomData;

//https://github.com/mattdesl/three-line-2d
//TODO try this.
//https://mattdesl.svbtle.com/drawing-lines-is-hard


pub mod circle_program;
pub mod vbo;


use glutin::NotCurrent;
use glutin::PossiblyCurrent;



pub struct GlSys{
    windowed_context:glutin::WindowedContext<PossiblyCurrent>,
}




impl GlSys{
    ///array should be full of xy pairs
    ///the orientation:
    ///0,0 is top left
    ///width,0 is top right
    ///0,height is bottom left
    ///width,height is bottom right
    pub fn new(events_loop:&glutin::event_loop::EventLoop<()>)->GlSys{

        let mut border=axgeom::Rect::new(0.0,0.0,0.0,0.0);
        let point_size=0.0;

        use glutin::window::Fullscreen;
        let fullscreen = Fullscreen::Borderless(prompt_for_monitor(events_loop));

        let gl_window = glutin::window::WindowBuilder::new()
            .with_fullscreen(Some(fullscreen));
         
        //we are targeting only opengl 3.0 es. and glsl 300 es.
        
        let windowed_context = glutin::ContextBuilder::new()
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGlEs, (3, 0)))
        .with_vsync(true)
        .build_windowed(gl_window,&events_loop).unwrap();


        std::thread::sleep(std::time::Duration::from_millis(500));
        
        let windowed_context = unsafe { windowed_context.make_current().unwrap() };

        let glutin::dpi::LogicalSize{width: _,height: _}=windowed_context.window().inner_size();

                
        let windowed_context = unsafe { windowed_context.make_current() }.unwrap();

        let glutin::dpi::LogicalSize{width,height}=windowed_context.window().inner_size();

        // It is essential to make the context current before calling `gl::load_with`.

        //let cs=ContextSetup::new(windowed_context.context(),width as u32,height as u32,border,point_size);

        use glutin::Context;

        // Load the OpenGL function pointers
        gl::load_with(|symbol| windowed_context.get_proc_address(symbol) as *const _);
        assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
        
        
        assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
        GlSys{windowed_context}

    }
    
    
    pub fn get_dim(&self)->Vec2<usize>{
        let glutin::dpi::LogicalSize{width,height}=self.windowed_context.window().inner_size();
        vec2(width as usize,height as usize)
    }

    pub fn swap_buffers(&mut self){
        self.windowed_context.swap_buffers().unwrap();
        assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
    }

}


use glutin::event::{
    ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent,
};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::monitor::{MonitorHandle, VideoMode};
use glutin::window::{Fullscreen, WindowBuilder};
use std::io::{stdin, stdout, Write};


// Enumerate monitors and prompt user to choose one
fn prompt_for_monitor(el: &EventLoop<()>) -> MonitorHandle {
    let num =0;
    let monitor = el
        .available_monitors()
        .nth(num)
        .expect("Please enter a valid ID");

    monitor
}

fn prompt_for_video_mode(monitor: &MonitorHandle) -> VideoMode {
    for (i, video_mode) in monitor.video_modes().enumerate() {
        println!("Video mode #{}: {}", i, video_mode);
    }

    print!("Please write the number of the video mode to use: ");
    stdout().flush().unwrap();

    let mut num = String::new();
    stdin().read_line(&mut num).unwrap();
    let num = num.trim().parse().ok().expect("Please enter a number");
    let video_mode = monitor
        .video_modes()
        .nth(num)
        .expect("Please enter a valid ID");

    println!("Using {}", video_mode);

    video_mode
}

