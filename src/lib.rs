
mod gl {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}


mod shader;
use shader::*;


extern crate glutin;
extern crate axgeom;

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
void main() {
    gl_PointSize = point_size;
    gl_Position = vec4(position*mmatrix, 0.0, 1.0);
}";

static FS_SRC: &'static str = "
#version 300 es
precision mediump float;
uniform vec3 bcol;
out vec4 out_color;
void main() {

    vec2 coord = gl_PointCoord - vec2(0.5);  //from [0,1] to [-0.5,0.5]
    float dis=dot(coord,coord);
    if(dis > 0.25)                  //outside of circle radius?
        discard;

    out_color = vec4(bcol, 1.0);
}";



#[repr(transparent)]
#[derive(Copy,Clone,Debug,Default)]
pub struct Vertex(pub [f32;2]);



use glutin::NotCurrent;
use glutin::PossiblyCurrent;



struct ContextSetup{
    program:GLuint,
    fs:GLuint,
    vs:GLuint,
    vbo:u32,
}

impl Drop for ContextSetup{
    fn drop(&mut self){
        // Cleanup
        unsafe {
            gl::DeleteProgram(self.program);
            gl::DeleteShader(self.fs);
            gl::DeleteShader(self.vs);
            gl::DeleteBuffers(1, &self.vbo);
        }
    }
}



impl ContextSetup{

    fn new(context:&glutin::Context<PossiblyCurrent>,width:u32,height:u32,verts:&[Vertex],game_world:Rect<f32>,point_size:f32)->ContextSetup{
        use glutin::Context;

        // Load the OpenGL function pointers
        gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);


        // Create GLSL shaders
        let vs = compile_shader(VS_SRC, gl::VERTEX_SHADER);
        let fs = compile_shader(FS_SRC, gl::FRAGMENT_SHADER);
        let program = link_program(vs, fs);

        let mut vbo = 0;

        unsafe {

            // Create a Vertex Buffer Object and copy the vertex data to it
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (verts.len() *mem::size_of::<Vertex>()) as GLsizeiptr,
                mem::transmute(verts.as_ptr()),
                gl::DYNAMIC_DRAW,
            );

            // Use shader program
            gl::UseProgram(program);
            
            gl::BindAttribLocation(program, 0, CString::new("out_color").unwrap().as_ptr());
            
            // Specify the layout of the vertex data
            let pos_attr = gl::GetAttribLocation(program, CString::new("position").unwrap().as_ptr());
            gl::EnableVertexAttribArray(pos_attr as GLuint);
            gl::VertexAttribPointer(
                pos_attr as GLuint,
                2,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                0,
                ptr::null(),
            );
        }
        
        Self::set_border_radius(program,game_world,width as usize,height as usize,point_size);

        ContextSetup{fs,vs,vbo,program}
    }

    fn set_border_radius(program:GLuint,game_world:Rect<f32>,width:usize,height:usize,point_size:f32){
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
            
            let myloc:GLint = gl::GetUniformLocation(program, CString::new("mmatrix").unwrap().as_ptr());
            gl::UniformMatrix2fv(myloc, 1, 0,std::mem::transmute(&matrix[0][0]));



            let point_size=point_size*(width/w);
            let myloc:GLint = gl::GetUniformLocation(program, CString::new("point_size").unwrap().as_ptr());
            gl::Uniform1f(myloc,point_size);
        }
    }

}



pub struct GlSys{
    buffer:Vec<Vertex>,
    windowed_context:glutin::WindowedContext<PossiblyCurrent>,
    cs:ContextSetup,
    back_col:[f32;3],
    _p:PhantomData<*mut usize>
}


impl GlSys{
    pub fn get_num_verticies(&self)->usize{
        self.buffer.len()
    }


    ///array should be full of xy pairs
    ///the orientation:
    ///0,0 is top left
    ///width,0 is top right
    ///0,height is bottom left
    ///width,height is bottom right
    pub fn new(events_loop:&glutin::EventsLoop)->GlSys{
        let num_verticies=0;
        let mut border=axgeom::Rect::new(0.0,0.0,0.0,0.0);
        let point_size=0.0;
        //let radius=game_response.new_game_world.unwrap().1;


        let mut buffer=Vec::new();
        buffer.resize(num_verticies,Vertex([0.0;2]));

        let size = glutin::dpi::PhysicalSize::new(1024., 768.);

        let gl_window = glutin::WindowBuilder::new().with_multitouch()
            .with_dimensions(glutin::dpi::LogicalSize::from_physical(size, 1.0));
 

        //we are targeting only opengl 3.0 es. and glsl 300 es.
        
        let windowed_context = glutin::ContextBuilder::new()
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGlEs, (3, 0)))
        .with_vsync(true).
        build_windowed(gl_window,&events_loop).unwrap();
  
        let windowed_context = unsafe { windowed_context.make_current().unwrap() };


        let glutin::dpi::LogicalSize{width: _,height: _}=windowed_context.window().get_inner_size().unwrap();

        use glutin::Context;
        
        let windowed_context = unsafe { windowed_context.make_current() }.unwrap();

        let glutin::dpi::LogicalSize{width,height}=windowed_context.window().get_inner_size().unwrap();
         // It is essential to make the context current before calling `gl::load_with`.
        
        
        let cs=ContextSetup::new(windowed_context.context(),width as u32,height as u32,&buffer,border,point_size);

        assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
        GlSys{windowed_context,buffer,cs,back_col:[0.,0.,0.],_p:PhantomData}

    }
    

    pub fn set_bot_color(&mut self,col:[f32;3]){
        unsafe{
            let myloc:GLint = gl::GetUniformLocation(self.cs.program, CString::new("bcol").unwrap().as_ptr());
      
            gl::Uniform3fv(myloc,1,std::mem::transmute(&col[0]));
            
        }
        assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
        
    }

    pub fn set_camera_and_bot_radius(&mut self,border:Rect<f32>,radius:f32){
        let (width,height)=self.get_dim();

        ContextSetup::set_border_radius(self.cs.program,border,width,height,radius);
        assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
        
    }
    pub fn set_back_color(&mut self,col:[f32;3]){
        self.back_col=col;
    }
    pub fn get_dim(&self)->(usize,usize){
        let glutin::dpi::LogicalSize{width,height}=self.windowed_context.window().get_inner_size().unwrap();
        (width as usize,height as usize)
    }

    
    pub fn update<T>(&mut self,arr:&[T],mut func:impl FnMut(&T)->Vertex){
        assert!(arr.len()==self.buffer.len());
        
        for (a,b) in self.buffer.iter_mut().zip(arr.iter()){
            *a=func(b);
        }
        unsafe{
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (self.buffer.len()*mem::size_of::<Vertex>()) as GLsizeiptr,
                mem::transmute(self.buffer.as_ptr()),
            );
        
        }
        assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
        
    }

    pub fn re_generate_buffer(&mut self,num_verticies:usize){
        self.buffer.resize(num_verticies,Vertex([0.0;2]));
        let _vbo=&mut self.cs.vbo;
        unsafe{

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.buffer.len() *mem::size_of::<Vertex>()) as GLsizeiptr,
                mem::transmute(self.buffer.as_ptr()),
                gl::DYNAMIC_DRAW,
            );
        }
        assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
        
    }
    
    pub fn draw(&self){
        use glutin::Context;
        unsafe{
            let b=self.back_col;
            // Clear the screen to black
            gl::ClearColor(b[0], b[1], b[2], 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::DrawArrays(gl::POINTS,0, self.buffer.len() as i32);
        }
        self.windowed_context.swap_buffers().unwrap();
        assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
        
    }
}


