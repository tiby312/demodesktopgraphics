use axgeom;
use axgeom::*;
use crate::shader::*;
use crate::gl;
use crate::gl::types::*;

use crate::vbo::Buffer;
use core::mem;

use std::ptr;
use std::str;
use std::ffi::CString;

// Shader sources
static VS_SRC: &'static str = "
#version 300 es
in vec2 position;
uniform mat3 mmatrix;
uniform float point_size;
in float alpha;
out float alpha2;
void main() {
    gl_PointSize = point_size;
    vec3 pp=vec3(position,0.0);
    gl_Position = vec4(pp.xyz*mmatrix, 1.0);
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





fn set_border_radius(program:GLuint,game_world:Rect<f32>,width:usize,height:usize,point_size:f32,square:bool){
    let width=width as f32;
    let _height=height as f32;

    let ((x1,x2),(y1,y2))=game_world.get();
    let w=x2-x1;
    let h=y2-y1;

    let scalex=2.0/w;
    let scaley=2.0/h;

    let tx=x1-(w/2.0);
    let ty=y1-(h/2.0);
    unsafe{
        let matrix= [
                [scalex, 0.0, tx],
                [0.0, -scaley,ty],
                [0.0,0.0,1.0]
            ];    
        

        gl::UseProgram(program);
        


        let point_size2=point_size*(width/w);
        
        //dbg!(width,w,point_size,point_size2);

        let myloc:GLint = gl::GetUniformLocation(program, CString::new("square").unwrap().as_ptr());
        assert_eq!(gl::GetError(),gl::NO_ERROR);
    
        let square=if square{1}else{0};
        gl::Uniform1i(myloc,square);
        assert_eq!(gl::GetError(),gl::NO_ERROR);
    

        let myloc:GLint = gl::GetUniformLocation(program, CString::new("point_size").unwrap().as_ptr());
        assert_eq!(gl::GetError(),gl::NO_ERROR);
    
        gl::Uniform1f(myloc,point_size2);
        assert_eq!(gl::GetError(),gl::NO_ERROR);
    
        let myloc:GLint = gl::GetUniformLocation(program, CString::new("mmatrix").unwrap().as_ptr());
        assert_eq!(gl::GetError(),gl::NO_ERROR);
    
        gl::UniformMatrix3fv(myloc,1, 0,std::mem::transmute(&matrix[0][0]));
        assert_eq!(gl::GetError(),gl::NO_ERROR);
    
    }
}




pub struct CircleProgram{
    program:GLuint,
    fs:GLuint,
    vs:GLuint,
    
}

impl CircleProgram{
    pub fn new()->CircleProgram{

        // Create GLSL shaders
        let vs = compile_shader(VS_SRC, gl::VERTEX_SHADER);
        assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
        
        let fs = compile_shader(FS_SRC, gl::FRAGMENT_SHADER);
        assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
        
        let program = link_program(vs, fs);

        assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
        
        //Self::set_border_radius(program,game_world,width as usize,height as usize,point_size,true);
        CircleProgram{program,fs,vs}
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


impl Drop for CircleProgram{
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




pub struct DrawSession<'a>{
    a:&'a mut CircleProgram,
    border:Rect<f32>,
}

impl<'a> DrawSession<'a>{

    pub fn draw_vbo_section(&mut self,dim:Vec2<usize>,buffer:&Buffer<Vertex>,start:usize,end:usize,color:[f32;3],radius:f32,square:bool){
        let (width,height) = (dim.x,dim.y);

        unsafe{
            set_border_radius(self.a.program,self.border,width,height,radius,square);
            
            assert_eq!(gl::GetError(),gl::NO_ERROR);

            // Clear the screen to black
            
            //TODO move this down more?
            gl::BindBuffer(gl::ARRAY_BUFFER, buffer.get_id());
            
            let myloc:GLint = gl::GetUniformLocation(self.a.program, CString::new("bcol").unwrap().as_ptr());
      
            gl::Uniform3fv(myloc,1,std::mem::transmute(&color[0]));
                    
            assert_eq!(gl::GetError(),gl::NO_ERROR);

            /////
            let pos_attr = gl::GetAttribLocation(self.a.program, CString::new("position").unwrap().as_ptr());
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
            
            
            let pos_attr = gl::GetAttribLocation(self.a.program, CString::new("alpha").unwrap().as_ptr());
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
    pub fn finish(self){
        self.a.windowed_context.swap_buffers().unwrap();
        assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
    }
    */
}

