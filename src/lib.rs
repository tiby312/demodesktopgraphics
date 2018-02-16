// Copyright 2015 Brendan Zabarauskas and the gl-rs developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod gl {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}



use std::env;
use std::fs::File;
use std::path::Path;



//extern crate gl;
extern crate glutin;

use gl::types::*;
use std::mem;
use std::ptr;
use std::str;
use std::ffi::CString;


// Shader sources
static VS_SRC: &'static str = "
#version 300 es
in vec2 position;
uniform mat4 mmatrix;
void main() {
    gl_Position = vec4(position, 0.0, 1.0)*mmatrix;
}";

static FS_SRC: &'static str = "
#version 300 es

precision mediump float;
uniform vec3 bcol;
out vec4 out_color;


void main() {
    out_color = vec4(bcol, 1.0);
}";


#[derive(Copy,Clone,Debug,Default)]
pub struct Vertex(pub [f32;2]);


fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    let shader;
    unsafe {
        shader = gl::CreateShader(ty);
        // Attempt to compile the shader
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(
                shader,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );
            panic!(
                "{}",
                str::from_utf8(&buf)
                    .ok()
                    .expect("ShaderInfoLog not valid utf8")
            );
        }
    }
    shader
}

fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);
        // Get the link status
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetProgramInfoLog(
                program,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );
            panic!(
                "{}",
                str::from_utf8(&buf)
                    .ok()
                    .expect("ProgramInfoLog not valid utf8")
            );
        }
        program
    }
}



pub struct GlSys{
    length:usize,
    gl_window:glutin::GlWindow,
    cs:ContextSetup,
    back_col:[f32;3]
}


pub struct GlSysBuilder{
    gl_window:glutin::GlWindow
}
impl GlSysBuilder{

    pub fn new(events_loop:&glutin::EventsLoop)->GlSysBuilder{
        use glutin::GlContext;

        let window = glutin::WindowBuilder::new().with_multitouch()/*.with_dimensions(800,400)*/;
 
        let context = glutin::ContextBuilder::new()
        //we are targeting only opengl 3.0 es. and glsl 300 es.
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGlEs, (3, 0)))
        .with_vsync(true);
        
        let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

        //println!("gl_window dim={:?}",gl_window.get_inner_size().unwrap());

        let (width,height)=gl_window.get_inner_size().unwrap();
        GlSysBuilder{gl_window}
    }

    pub fn get_dim(&self)->(usize,usize){
        let (width,height)=self.gl_window.get_inner_size().unwrap();
        (width as usize,height as usize)
    }
}


struct ContextSetup{
    program:GLuint,
    fs:GLuint,
    vs:GLuint,
    vbo:u32,
    vao:u32,
}

impl Drop for ContextSetup{
    fn drop(&mut self){
        // Cleanup
        unsafe {
            gl::DeleteProgram(self.program);
            gl::DeleteShader(self.fs);
            gl::DeleteShader(self.vs);
            gl::DeleteBuffers(1, &self.vbo);
            //TODO what to replace with?
            //gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}
impl ContextSetup{
    fn new(gl_window:&glutin::GlWindow,width:u32,height:u32,verts:&[Vertex])->ContextSetup{
        use glutin::GlContext;

        // Load the OpenGL function pointers
        // TODO: `as *const _` will not be needed once glutin is updated to the latest gl version
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);


        // Create GLSL shaders
        let vs = compile_shader(VS_SRC, gl::VERTEX_SHADER);
        let fs = compile_shader(FS_SRC, gl::FRAGMENT_SHADER);
        let program = link_program(vs, fs);

        //println!("created vertex program");
        let mut vao = 0;
        let mut vbo = 0;

        unsafe {
            // Create Vertex Array Object
            //gl::GenVertexArrays(1, &mut vao);
            //gl::BindVertexArray(vao);

            // Create a Vertex Buffer Object and copy the vertex data to it
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (verts.len() *2* mem::size_of::<GLfloat>()) as GLsizeiptr,
                mem::transmute(&verts[0]),
                gl::DYNAMIC_DRAW,
            );

            //println!("created buffer draw");

            // Use shader program
            gl::UseProgram(program);
            //println!("used program");
            //gl::BindFragDataLocation(program, 0, CString::new("out_color").unwrap().as_ptr());
            gl::BindAttribLocation(program, 0, CString::new("out_color").unwrap().as_ptr());
            
            //println!("bind frag");
            // Specify the layout of the vertex data
            let pos_attr = gl::GetAttribLocation(program, CString::new("position").unwrap().as_ptr());
            //println!("attrib location");
            gl::EnableVertexAttribArray(pos_attr as GLuint);
            //println!("enabled");
            gl::VertexAttribPointer(
                pos_attr as GLuint,
                2,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                0,
                ptr::null(),
            );

            //println!("creaed attrib");
        }
        let scalex = 2.0 * (1.0 / width as f32); //TODO don't hard code!!!!
        let scaley = 2.0 * (1.0 / height as f32);
        //let scalex=1.0;
        //let scaley=1.0;
        unsafe{
            
            let matrix= [
                    [scalex, 0.0, 0.0, -1.0],
                    [0.0, -scaley, 0.0, 1.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0f32]
                ];
            
            
            let myLoc:GLint = gl::GetUniformLocation(program, CString::new("mmatrix").unwrap().as_ptr());
            //println!("got uniform");
            gl::UniformMatrix4fv(myLoc, 1, 0,std::mem::transmute(&matrix[0][0]));
            //println!("set uniform matrix");
        }
        /*
        unsafe{
            let myLoc:GLint = gl::GetUniformLocation(program, CString::new("bcol").unwrap().as_ptr());
      
            let mut arr=[1.0,0.5,1.0f32];
            gl::Uniform3fv(myLoc,1,std::mem::transmute(&arr[0]));
            
        }
        */
        /*
        unsafe{
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable( gl::BLEND );
        }
        */
        

        ContextSetup{fs,vs,vbo,vao,program}
    }

}
impl GlSys{

    ///array should be full of xy pairs
    ///the orientation:
    ///0,0 is top left
    ///width,0 is top right
    ///0,height is bottom left
    ///width,height is bottom right
    pub fn new(builder:GlSysBuilder,verts:&[Vertex])->GlSys{
        //println!("verts len is ={}",verts.len());
        use glutin::GlContext;
        let GlSysBuilder{gl_window}=builder;
        let (width,height)=gl_window.get_inner_size().unwrap();
         // It is essential to make the context current before calling `gl::load_with`.
        
        unsafe { gl_window.make_current() }.unwrap();

        let cs=ContextSetup::new(&gl_window,width,height,verts);

        //Self::update_uniform(program,&gl_window,width,height);
        //println!("updated uniform");
        GlSys{gl_window,length:verts.len(),cs,back_col:[0.2,0.2,0.2]}

    }
    

    pub fn set_bot_color(&mut self,col:[f32;3]){
        unsafe{
            let myLoc:GLint = gl::GetUniformLocation(self.cs.program, CString::new("bcol").unwrap().as_ptr());
      
            //let mut arr=[1.0,0.5,1.0f32];
            gl::Uniform3fv(myLoc,1,std::mem::transmute(&col[0]));
            
        }
    }
    pub fn set_back_color(&mut self,col:[f32;3]){
        self.back_col=col;
    }
    pub fn get_dim(&self)->(usize,usize){
        let (width,height)=self.gl_window.get_inner_size().unwrap();
        (width as usize,height as usize)
    }
    pub fn update(&self,verts:&[Vertex]){
        assert!(verts.len()==self.length);
        unsafe{
            if verts.len()>0{
                gl::BufferSubData(
                    gl::ARRAY_BUFFER,
                    0,
                    (verts.len()*2 * mem::size_of::<GLfloat>()) as GLsizeiptr,
                    mem::transmute(&verts[0]),
                );
            }
        }
    }

    pub fn re_generate_buffer(&mut self,verts:&[Vertex]){
        self.length=verts.len();
        let vbo=&mut self.cs.vbo;
        unsafe{

            if verts.len()>0{
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (verts.len() *2* mem::size_of::<GLfloat>()) as GLsizeiptr,
                    mem::transmute(&verts[0]),
                    gl::DYNAMIC_DRAW,
                );
            }
        }
    }
    
    pub fn draw(&self){
        use glutin::GlContext;
        unsafe{
            let b=self.back_col;
            // Clear the screen to black
            gl::ClearColor(b[0], b[1], b[2], 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);


            // Draw a triangle from the 3 vertices
            gl::DrawArrays(gl::TRIANGLES, 0, self.length as i32 *2);
        }
        self.gl_window.swap_buffers().unwrap();
    }
}


/*
fn main() {

    // Vertex data
    let mut verts: [GLfloat; 6] = [0.0, 0.0, 1024.0, 0.0, 1024.0,768.0];
    let verts:&mut [Vertex;3]=unsafe{std::mem::transmute(&mut verts)};

    

    let mut events_loop = glutin::EventsLoop::new();

    
    let j=GlSysBuilder::new(&events_loop);
    
    let k=GlSys::new(j,verts);    
    

    let mut val=0.5;
    loop{
        events_loop.poll_events(|event| {
            use glutin::{ControlFlow, Event, WindowEvent};

            if let Event::WindowEvent { event, .. } = event {
                if let WindowEvent::Closed = event {
                    //return ControlFlow::Break;
                }
            }

        });
        val=0.5-val;
        verts[0].0[0]=val;
        k.update(verts);
        k.draw();
    }
}*/




