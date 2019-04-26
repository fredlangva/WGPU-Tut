extern crate wgpu;
extern crate tobj;


use wgpu::*;
use wgpu::winit::{
    EventsLoop, WindowBuilder, WindowEvent, Event, dpi::LogicalSize,
    KeyboardInput, VirtualKeyCode, ElementState };
use std::path::{Path, PathBuf};
use glsl_to_spirv;
use cgmath;
use std::collections::HashMap;
use std::fs::{read_to_string};
use std::io::{Read};
use std::mem;


#[derive(Debug, Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    normals: [f32; 3],
    texcoords: [f32; 2],
}


#[allow(dead_code)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

fn load_glsl(name: &str, stage: ShaderStage) -> Vec<u8> {
        let ty = match stage {
        ShaderStage::Vertex => glsl_to_spirv::ShaderType::Vertex,
        ShaderStage::Fragment => glsl_to_spirv::ShaderType::Fragment,
        ShaderStage::Compute => glsl_to_spirv::ShaderType::Compute,
    };
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join(name);
    let code = match read_to_string(&path) {
        Ok(code) => code,
        Err(e) => panic!("Unable to read {:?}: {:?}", path, e),
    };

    let mut output = glsl_to_spirv::compile(&code, ty).unwrap();
    let mut spv = Vec::new();
    output.read_to_end(&mut spv).unwrap();
    spv
}

fn main() {
    println!("Hello, world!");
    let width = 1024.0;
    let height = 768.0;
    let mut events_loop = EventsLoop::new();

    let _window = WindowBuilder::new()
        .with_title("WGPU")
        .with_dimensions(LogicalSize::new(width as f64, height as f64))
        .build(&events_loop)
        .unwrap();
    let dpi_factor = _window.get_hidpi_factor();
    let size = _window
        .get_inner_size()
        .unwrap()
        .to_physical(dpi_factor);

// ------------------------------------------------
// Graphics stuff
// 1. Get an object :)
    let path = format!(env!("CARGO_MANIFEST_DIR"));
    let assets_file = format!("{}/House1/house.obj", path);
    println!("Asset file: {}", assets_file);
// load the vertex data Vec
//    let mut vertex_data = Vec::new();
    let (models, materials) = match tobj::load_obj(Path::new(&assets_file)) {
        Ok((models, materials)) => {
/*            
            let mut index = Vec::new();
            for m in models {
                let mesh = m.mesh;
                println!("uploading model: {}", m.name);
                for idx in &mesh.indices {
                    let i = *idx as usize;
                    let pos = [
                        mesh.positions[3 * i] * 0.3,
                        mesh.positions[3 * i + 1] * 0.3,
                        mesh.positions[3 * i + 2] * 0.3,
                    ];
                    let norm = if !mesh.normals.is_empty() {
                        [
                            mesh.normals[3 * i],
                            mesh.normals[3 * i + 1],
                            mesh.normals[3 * i + 2],
                        ]
                    } else {
                        [0.0, 0.0, 0.0]
                    };
                    let tex = [
                        mesh.texcoords[2 * i],
                        mesh.texcoords[2 * i + 1],
                    ];
                    vertex_data.push(Vertex {
                        position: pos,
                        normals: norm,
                        texcoords: tex,
                    })
                }
                index.push(mesh.indices);
            }
*/            
            (models, materials)
       },
        Err(e) => {
            println!("Failed to load {:?} due to {:?}", &assets_file, e);
            panic!("Ugg");
        },
    };

    let mut vertex_data = Vec::new();
    for m in &models {
        let mesh = &m.mesh;
        println!("uploading model: {}", m.name);
        for idx in &mesh.indices {
            let i = *idx as usize;
            let pos = [
                mesh.positions[3 * i] * 0.3,
                mesh.positions[3 * i + 1] * 0.3,
                mesh.positions[3 * i + 2] * 0.3,
            ];
            let norm = if !mesh.normals.is_empty() {
                [
                    mesh.normals[3 * i],
                    mesh.normals[3 * i + 1],
                    mesh.normals[3 * i + 2],
                ]
            } else {
                [0.0, 0.0, 0.0]
            };
            let tex = [
                mesh.texcoords[2 * i],
                mesh.texcoords[2 * i + 1],
            ];
            vertex_data.push(Vertex {
                position: pos,
                normals: norm,
                texcoords: tex,
            })
        }

    }


//    let vertex_data = &models[0].mesh.positions;
    let index_data = &models[0].mesh.indices;

    println!("Vertex data loaded");
// 2. Set up the Graphics device
    let instance = wgpu::Instance::new();
    let adapter = instance.get_adapter(&wgpu::AdapterDescriptor {
        power_preference: wgpu::PowerPreference::LowPower,
    });
    let mut device = adapter.create_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions {
            anisotropic_filtering: false,
        },
    });
// 3. Set up the drawing Surface
    let surface = instance.create_surface(&_window);
// 5. Setup the Vertex and Index buffers

    let vertex_buf = device
        .create_buffer_mapped(vertex_data.len(), wgpu::BufferUsageFlags::VERTEX)
        .fill_from_slice(&vertex_data);
    let vertex_size = mem::size_of::<Vertex>();
    println!("loaded vertex buffer");


    let index_buf = device
        .create_buffer_mapped(index_data.len(), wgpu::BufferUsageFlags::INDEX)
        .fill_from_slice(&index_data);
    let index_count = index_data.len();
    println!("Loaded Index buffer");

// 6. Define Swap Chain Descriptor
    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsageFlags::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8Unorm,
        width: size.width.round() as u32,
        height: size.height.round() as u32,
    };

// 7. Create Uniform Buffers  - we need to transform the Vertex coordinates to a 
//      set that is between 1 and -1 for all points
    let aspect_ratio =  1024.0 / 768.0;



// 8. Define the BindGroup and PipelineLayout
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        bindings: &[
/*            wgpu::BindGroupLayoutBinding {
                binding: 0,
                visibility: wgpu::ShaderStageFlags::VERTEX,
                ty: wgpu::BindingType::UniformBuffer,
            },
*/
        ]
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        bindings: &[
/*            wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &uniform_buf,
                    range: 0..64,
                },
            },
*/
        ]
    });
    println!("After Bind Stuff");

// 4. Get the Shaders
    let vs_bytes = load_glsl("test.vert", ShaderStage::Vertex);
    let fs_bytes = load_glsl("test.frag", ShaderStage::Fragment);
    let vs_module = device.create_shader_module(&vs_bytes);
    let fs_module = device.create_shader_module(&fs_bytes);
    println!("After shader load");

// 7. Create the Render Pipeline

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        layout: &pipeline_layout,
        vertex_stage: wgpu::PipelineStageDescriptor {
            module: &vs_module,
            entry_point: "main",
        },
        fragment_stage: wgpu::PipelineStageDescriptor {
            module: &fs_module,
            entry_point: "main",
        },
        rasterization_state: wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::None,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        },
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[wgpu::ColorStateDescriptor {
            format: wgpu::TextureFormat::Bgra8Unorm,
            color: wgpu::BlendDescriptor::REPLACE,
            alpha: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWriteFlags::ALL,
        }],
        depth_stencil_state: None,
        index_format: wgpu::IndexFormat::Uint16,
        vertex_buffers: &[wgpu::VertexBufferDescriptor {
            stride: vertex_size as u32,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    attribute_index: 0,
                    format: wgpu::VertexFormat::Float3,  // Vec is 3 f32 for Position
                    offset: 0,
                },
                wgpu::VertexAttributeDescriptor {
                    attribute_index: 1,
                    format: wgpu::VertexFormat::Float3, // Vec is f32 for Noraml
                    offset: 4 * 3,
                },
                wgpu::VertexAttributeDescriptor {
                    attribute_index: 2,
                    format: wgpu::VertexFormat::Float2, // Vec is f32 for Noraml
                    offset: 4 * 3,
                },
            ],
        }],
        sample_count: 1,
    });
    println!("After render pipeline create");


    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);
    println!("After Swap chain create");
    let clear_color = wgpu::Color { r: 0.0, g: 0.5, b: 0.5, a: 1.0 };

// ------------------------------------------------
// Main loop
    let mut running = true;
//    let mut resized_extent = None;
    while running {
        events_loop.poll_events(|ev| {
            match ev {
                Event::WindowEvent { event: WindowEvent::KeyboardInput {
                        // Escape key to exit 
                        input: KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            state: ElementState::Pressed,
                            .. 
                        },
                        ..
                    },
                    ..    
                } |
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    running = false
                },     
/*                          
                Event::WindowEvent { event: WindowEvent::Resized(_), .. } => {
                    let dpi_factor = _window.get_hidpi_factor();
                    let size = _window
                        .get_inner_size()
                        .unwrap()
                        .to_physical(dpi_factor);
                    resized_extent = Some( wgpu::Extent3d {
                        width: size.width.round() as u32,
                        height: size.height.round() as u32,
                        depth: 1,
                    });
                    let mut sc_desc = wgpu::SwapChainDescriptor {
                        usage: wgpu::TextureUsageFlags::OUTPUT_ATTACHMENT,
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        width: size.width.round() as u32,
                        height: size.height.round() as u32,
                    };
                    let _swap_chain = device.create_swap_chain(&surface, &sc_desc);
                },
                // add additional stuff here like more keys or mouse
*/
                _ => (),
            }
        });

        let frame = swap_chain.get_next_texture();
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: clear_color,
                }],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&render_pipeline);
            rpass.set_bind_group(0, &bind_group);
            rpass.set_index_buffer(&index_buf, 0);
            rpass.set_vertex_buffers(&[(&vertex_buf, 0)]);

            rpass.draw_indexed(0..index_count as u32, 0, 0..1);
        }

        device.get_queue().submit(&[encoder.finish()]);

    };
    println!("Done!");
    

}
