
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


// Shader sources
static VS_SRC: &'static str = "
#version 300 es
in vec2 position;
uniform mat2 mmatrix;
uniform float point_size;
in float alpha;
out float alpha2;
void main() {
    gl_PointSize = point_size;
    gl_Position = vec4(position.xy*mmatrix, 0.0, 1.0);
    alpha2=alpha;
}";



//https://blog.lapingames.com/draw-circle-glsl-shader/
static FS_SRC: &'static str = "
#version 300 es
precision mediump float;
in float alpha2;
uniform vec3 bcol;
uniform bool square;
out vec4 out_color;
void main() {

    vec2 coord = gl_PointCoord - vec2(0.5);
    if (!square){
        float dis=dot(coord,coord);
        if(dis > 0.25)                  //outside of circle radius?
            discard;
    }

    out_color = vec4(bcol,alpha2);
}";




#[repr(transparent)]
#[derive(Copy,Clone,Debug,Default)]
pub struct Vertex(pub [f32;3]);



use glutin::NotCurrent;
use glutin::PossiblyCurrent;



struct ContextSetup{
    program:GLuint,
    fs:GLuint,
    vs:GLuint,
}

impl Drop for ContextSetup{
    fn drop(&mut self){
        println!("dropping ");
        // Cleanup
        unsafe {
            gl::DeleteProgram(self.program);
            gl::DeleteShader(self.fs);
            gl::DeleteShader(self.vs);
        }
    }
}



impl ContextSetup{

    fn new(context:&glutin::Context<PossiblyCurrent>,width:u32,height:u32,game_world:Rect<f32>,point_size:f32)->ContextSetup{
        use glutin::Context;

        // Load the OpenGL function pointers
        gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);
        assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
        

        // Create GLSL shaders
        let vs = compile_shader(VS_SRC, gl::VERTEX_SHADER);
        assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
        
        let fs = compile_shader(FS_SRC, gl::FRAGMENT_SHADER);
        assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
        
        let program = link_program(vs, fs);

        assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
        
        Self::set_border_radius(program,game_world,width as usize,height as usize,point_size,true);
        

        ContextSetup{fs,vs,program}
    }

    fn set_border_radius(program:GLuint,game_world:Rect<f32>,width:usize,height:usize,point_size:f32,square:bool){
        let width=width as f32;
        let _height=height as f32;

        let ((x1,x2),(y1,y2))=game_world.get();
        let w=x2-x1;
        let h=y2-y1;

        let scalex=2.0/w;
        let scaley=2.0/h;

        unsafe{
            let matrix= [
                    [scalex, 0.0, ],
                    [0.0, -scaley,],
                ];    
            

            gl::UseProgram(program);
            


            let point_size2=point_size*(width/w);
            
            //dbg!(width,w,point_size,point_size2);

            let myloc:GLint = gl::GetUniformLocation(program, CString::new("square").unwrap().as_ptr());
            assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
        
            let square=if square{1}else{0};
            gl::Uniform1i(myloc,square);
            assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
        

            let myloc:GLint = gl::GetUniformLocation(program, CString::new("point_size").unwrap().as_ptr());
            assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
        
            gl::Uniform1f(myloc,point_size2);
            assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
        
            let myloc:GLint = gl::GetUniformLocation(program, CString::new("mmatrix").unwrap().as_ptr());
            assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
        
            gl::UniformMatrix2fv(myloc,1, 0,std::mem::transmute(&matrix[0][0]));
            assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
        
        }
    }

}



pub struct GlSys{
    windowed_context:glutin::WindowedContext<PossiblyCurrent>,
    cs:ContextSetup,
    _p:PhantomData<*mut usize>
}


#[derive(Clone,Debug)]
pub struct Buffer{
    vbo:u32,
    buffer:Vec<Vertex>
}

impl Drop for Buffer{
    fn drop(&mut self){
        //TODO make sure this is ok to do
        unsafe{
            gl::DeleteBuffers(1, &self.vbo);
        }
    }
}


impl Buffer{
    pub fn get_verts_mut(&mut self)->&mut [Vertex]{
        &mut self.buffer
    }

    
    pub fn update(&mut self){
        let vbo=&mut self.vbo;
        
        unsafe{
            gl::BindBuffer(gl::ARRAY_BUFFER, *vbo);
            
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (self.buffer.len()*mem::size_of::<Vertex>()) as GLsizeiptr,
                mem::transmute(self.buffer.as_ptr()),
            );
        }
        assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);   
    }
    

    pub fn get_num_verticies(&self)->usize{
        self.buffer.len()
    }

    
    pub fn re_generate_buffer(&mut self,num_verticies:usize){
        
        self.buffer.resize(num_verticies,Vertex([0.0;3]));
        let vbo=&mut self.vbo;
        unsafe{
            gl::BindBuffer(gl::ARRAY_BUFFER, *vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.buffer.len() *mem::size_of::<Vertex>()) as GLsizeiptr,
                mem::transmute(self.buffer.as_ptr()),
                gl::DYNAMIC_DRAW,
            );
        }
        assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
        
    }

    pub fn create_vbo(num_verticies:usize)->Buffer{
        let mut vbo = 0;
        
        let mut buffer=Vec::new();
        buffer.resize(num_verticies,Vertex([0.0;3]));
        
        unsafe {

            // Create a Vertex Buffer Object and copy the vertex data to it
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (buffer.len() *mem::size_of::<Vertex>()) as GLsizeiptr,
                mem::transmute(buffer.as_ptr()),
                gl::DYNAMIC_DRAW,
            );

            
        }

        Buffer{vbo,buffer}
    }
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

        use glutin::Context;
                
        let windowed_context = unsafe { windowed_context.make_current() }.unwrap();

        let glutin::dpi::LogicalSize{width,height}=windowed_context.window().inner_size();

        // It is essential to make the context current before calling `gl::load_with`.

        let cs=ContextSetup::new(windowed_context.context(),width as u32,height as u32,border,point_size);


        
        assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
        GlSys{windowed_context,cs,_p:PhantomData}

    }
    
    
    pub fn get_dim(&self)->(usize,usize){
        let glutin::dpi::LogicalSize{width,height}=self.windowed_context.window().inner_size();
        (width as usize,height as usize)
    }


    pub fn new_draw_session(&mut self,back_color:[f32;3],border:Rect<f32>)->DrawSession{
        
        
        unsafe{
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable( gl::BLEND );

            gl::ClearColor(back_color[0], back_color[1], back_color[2], 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        DrawSession{a:self,border}
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


pub struct DrawSession<'a>{
    a:&'a mut GlSys,
    border:Rect<f32>,
}

impl<'a> DrawSession<'a>{

    pub fn draw_vbo_section(&mut self,buffer:&Buffer,start:usize,end:usize,color:[f32;3],radius:f32,square:bool){
        let (width,height) = self.a.get_dim();

        unsafe{
            ContextSetup::set_border_radius(self.a.cs.program,self.border,width,height,radius,square);
            assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);

            // Clear the screen to black
            
            gl::BindBuffer(gl::ARRAY_BUFFER, buffer.vbo);
            
            let myloc:GLint = gl::GetUniformLocation(self.a.cs.program, CString::new("bcol").unwrap().as_ptr());
      
            gl::Uniform3fv(myloc,1,std::mem::transmute(&color[0]));
                    
            assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);

            /////
            let pos_attr = gl::GetAttribLocation(self.a.cs.program, CString::new("position").unwrap().as_ptr());
            gl::EnableVertexAttribArray(pos_attr as GLuint);
            gl::VertexAttribPointer(
                pos_attr as GLuint,
                2,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                3*mem::size_of::<f32>() as i32,
                ptr::null(),
            );
            /////
            
            
            let pos_attr = gl::GetAttribLocation(self.a.cs.program, CString::new("alpha").unwrap().as_ptr());
            gl::EnableVertexAttribArray(pos_attr as GLuint);
            gl::VertexAttribPointer(
                pos_attr as GLuint,
                1,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                3*mem::size_of::<f32>() as i32,
                (2*mem::size_of::<f32>()) as *const std::ffi::c_void,
            );
            
            //////
            gl::DrawArrays(gl::POINTS,start as i32, end as i32);
        }
    }
    /*
    pub fn draw_vbo(&mut self,buffer:&Buffer,color:[f32;3]){
         unsafe{
            // Clear the screen to black
            
            gl::BindBuffer(gl::ARRAY_BUFFER, buffer.vbo);
            
            let myloc:GLint = gl::GetUniformLocation(self.a.cs.program, CString::new("bcol").unwrap().as_ptr());
      
            gl::Uniform3fv(myloc,1,std::mem::transmute(&color[0]));
                    
            assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);

            /////
            let pos_attr = gl::GetAttribLocation(self.a.cs.program, CString::new("position").unwrap().as_ptr());
            gl::EnableVertexAttribArray(pos_attr as GLuint);
            gl::VertexAttribPointer(
                pos_attr as GLuint,
                3,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                0,
                ptr::null(),
            );
            /////

            gl::DrawArrays(gl::POINTS,0, buffer.buffer.len() as i32);
        }
    }
    */
    pub fn finish(self){
        self.a.windowed_context.swap_buffers().unwrap();
        assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
    }
}

