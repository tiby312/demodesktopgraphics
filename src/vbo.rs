use axgeom;
use axgeom::*;
use crate::shader::*;
use crate::gl;
use crate::gl::types::*;

use std::ptr;
use std::str;
use std::ffi::CString;
use core::mem;

#[derive(Clone,Debug)]
pub struct Buffer<V>{
    vbo:u32,
    buffer:Vec<V>
}

impl<V> Drop for Buffer<V>{
    fn drop(&mut self){
        //TODO make sure this is ok to do
        unsafe{
            gl::DeleteBuffers(1, &self.vbo);
        }
    }
}


impl<V:Default> Buffer<V>{
    pub fn get_id(&self)->u32{
        self.vbo
    }
    pub fn get_verts_mut(&mut self)->&mut [V]{
        &mut self.buffer
    }

    
    pub fn update(&mut self){
        let vbo=&mut self.vbo;
        
        unsafe{
            gl::BindBuffer(gl::ARRAY_BUFFER, *vbo);
            
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (self.buffer.len()*mem::size_of::<V>()) as GLsizeiptr,
                mem::transmute(self.buffer.as_ptr()),
            );
        }
        assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);   
    }
    

    pub fn get_num_verticies(&self)->usize{
        self.buffer.len()
    }

    
    pub fn re_generate_buffer(&mut self,num_verticies:usize){
        
        self.buffer.resize_with(num_verticies,Default::default);
        let vbo=&mut self.vbo;
        unsafe{
            gl::BindBuffer(gl::ARRAY_BUFFER, *vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.buffer.len() *mem::size_of::<V>()) as GLsizeiptr,
                mem::transmute(self.buffer.as_ptr()),
                gl::DYNAMIC_DRAW,
            );
        }
        assert_eq!(unsafe{gl::GetError()},gl::NO_ERROR);
        
    }

    pub fn create_vbo(num_verticies:usize)->Buffer<V>{
        let mut vbo = 0;
        
        let mut buffer=Vec::new();
        buffer.resize_with(num_verticies,Default::default);
        
        unsafe {

            // Create a Vertex Buffer Object and copy the vertex data to it
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (buffer.len() *mem::size_of::<V>()) as GLsizeiptr,
                mem::transmute(buffer.as_ptr()),
                gl::DYNAMIC_DRAW,
            );

            
        }

        Buffer{vbo,buffer}
    }
}