// +x right
// +y up
// +z away from camera

use crate::compute::MipmapCompute;
use bve::load::mesh::Vertex as MeshVertex;
use cgmath::{
    Array, EuclideanSpace, InnerSpace, Matrix3, Matrix4, MetricSpace, Point3, Rad, SquareMatrix, Vector2, Vector3,
    Vector4,
};
use image::{Rgba, RgbaImage};
use indexmap::map::IndexMap;
use itertools::Itertools;
use num_traits::{ToPrimitive, Zero};
use std::{cmp::Ordering, io, mem::size_of};
use wgpu::*;
use winit::{dpi::PhysicalSize, window::Window};
use zerocopy::{AsBytes, FromBytes};

macro_rules! include_shader {
    (vert $name:literal) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/", $name, ".vs.spv"));
    };
    (geo $name:literal) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/", $name, ".gs.spv"));
    };
    (frag $name:literal) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/", $name, ".fs.spv"));
    };
    (comp $name:literal) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/", $name, ".cs.spv"));
    };
}

mod compute;

#[repr(C)]
#[derive(Clone, Copy, AsBytes, FromBytes)]
pub struct Vertex {
    _pos: [f32; 3],
    _normal: [f32; 3],
    _color: [f32; 4],
    _tex_coord: [f32; 2],
}

#[repr(C)]
#[derive(AsBytes)]
pub struct Uniforms {
    _matrix: [f32; 16],
    _transparent: u32,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ObjectHandle(u64);

struct Object {
    vertex_buffer: Buffer,

    index_buffer: Buffer,
    index_count: u32,

    texture: u64,

    uniform_buffer: Buffer,
    bind_group: BindGroup,

    location: Vector3<f32>,
    mesh_center_offset: Vector3<f32>,
    camera_distance: f32,

    transparent: bool,
    mesh_transparent: bool,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TextureHandle(u64);

struct Texture {
    texture_view: TextureView,
    transparent: bool,
}

struct Camera {
    location: Vector3<f32>,
    /// radians
    pitch: f32,
    /// radians
    yaw: f32,
}

impl Camera {
    pub fn compute_matrix(&self) -> Matrix4<f32> {
        // This is pre z-inversion, so z is flipped here
        let look_offset = Matrix3::from_diagonal(Vector3::new(1.0, 1.0, -1.0))
            * Matrix3::from_axis_angle(Vector3::unit_y(), Rad(self.yaw))
            * Matrix3::from_axis_angle(Vector3::unit_x(), Rad(self.pitch))
            * Vector3::unit_z();

        Matrix4::from_diagonal(Vector4::new(1.0, 1.0, -1.0, 1.0))
            * Matrix4::look_at(
                Point3::from_vec(self.location),
                Point3::from_vec(self.location + look_offset),
                Vector3::unit_y(),
            )
    }
}

pub struct Renderer {
    objects: IndexMap<u64, Object>,
    object_handle_count: u64,

    textures: IndexMap<u64, Texture>,
    texture_handle_count: u64,

    camera: Camera,
    screen_size: PhysicalSize<u32>,
    samples: MSAASetting,

    surface: Surface,
    device: Device,
    queue: Queue,
    swapchain: SwapChain,
    framebuffer: TextureView,
    depth_buffer: TextureView,
    opaque_pipeline: RenderPipeline,
    alpha_pipeline: RenderPipeline,
    pipeline_layout: PipelineLayout,
    bind_group_layout: BindGroupLayout,
    sampler: Sampler,

    vert_shader: ShaderModule,
    frag_shader: ShaderModule,

    mip_creator: compute::MipmapCompute,

    command_buffers: Vec<CommandBuffer>,
}

pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0, //
    0.0, -1.0, 0.0, 0.0, //
    0.0, 0.0, 0.5, 0.0, //
    0.0, 0.0, 0.5, 1.0,
);

fn generate_matrix(mx_view: &Matrix4<f32>, location: Vector3<f32>, aspect_ratio: f32) -> Matrix4<f32> {
    let mx_projection = cgmath::perspective(cgmath::Deg(55f32), aspect_ratio, 0.1, 1000.0);
    let mx_model = Matrix4::from_translation(location);
    OPENGL_TO_WGPU_MATRIX * mx_projection * mx_view * mx_model
}

fn convert_mesh_verts_to_verts(verts: Vec<MeshVertex>, mut indices: Vec<u32>) -> (Vec<Vertex>, Vec<u32>) {
    // First add the extra faces due to doubling
    let mut extra_indices = Vec::new();

    for (&i1, &i2, &i3) in indices.iter().tuples() {
        let v1_double = verts[i1 as usize].double_sided;
        let v2_double = verts[i2 as usize].double_sided;
        let v3_double = verts[i3 as usize].double_sided;

        if v1_double || v2_double || v3_double {
            extra_indices.push(i3);
            extra_indices.push(i2);
            extra_indices.push(i1);
        }
    }

    // Then convert the verts to the new format
    let out_verts = verts
        .into_iter()
        .map(|v| Vertex {
            _pos: v.position.into(),
            _color: v.color.into(),
            _normal: v.normal.into(),
            _tex_coord: v.coord.into(),
        })
        .collect_vec();

    indices.extend(extra_indices.into_iter());

    (out_verts, indices)
}

fn is_mesh_transparent(mesh: &[MeshVertex]) -> bool {
    mesh.iter().any(|v| v.color.w != 0.0 && v.color.w != 1.0)
}

fn is_texture_transparent(texture: &RgbaImage) -> bool {
    texture.pixels().any(|&Rgba([_, _, _, a])| a != 0 && a != 255)
}

fn find_mesh_center(mesh: &[Vertex]) -> Vector3<f32> {
    let first = if let Some(first) = mesh.first() {
        *first
    } else {
        return Vector3::zero();
    };
    // Bounding box time baby!
    let mut max: Vector3<f32> = first._pos.into();
    let mut min: Vector3<f32> = first._pos.into();

    for vert in mesh.iter().skip(1) {
        let pos: Vector3<f32> = vert._pos.into();
        max = max.zip(pos, |left, right| left.max(right));
        min = min.zip(pos, |left, right| left.min(right));
    }

    (max + min) / 2.0
}

fn mip_levels(size: Vector2<impl ToPrimitive>) -> u32 {
    let float_size = size.map(|v| v.to_f32().unwrap());
    let shortest = float_size.x.min(float_size.y);
    let mips = shortest.log2().floor();
    (mips as u32) + 1
}

struct MipIterator {
    count: u32,
    size: Vector2<u32>,
}

impl Iterator for MipIterator {
    type Item = (u32, Vector2<u32>);

    fn next(&mut self) -> Option<Self::Item> {
        self.size /= 2;
        self.count += 1;
        if self.size.x.is_zero() | self.size.y.is_zero() {
            None
        } else {
            Some((self.count, self.size))
        }
    }
}

fn enumerate_mip_levels(size: Vector2<impl ToPrimitive>) -> MipIterator {
    MipIterator {
        count: 0,
        size: size.map(|v| v.to_u32().unwrap()),
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum PipelineType {
    Normal,
    Alpha,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum MSAASetting {
    X1 = 1,
    X2 = 2,
    X4 = 4,
    X8 = 8,
}

impl MSAASetting {
    pub fn increment(self) -> Self {
        match self {
            Self::X1 => Self::X2,
            Self::X2 => Self::X4,
            _ => Self::X8,
        }
    }

    pub fn decrement(self) -> Self {
        match self {
            Self::X8 => Self::X4,
            Self::X4 => Self::X2,
            _ => Self::X1,
        }
    }
}

fn create_pipeline(
    device: &Device,
    layout: &PipelineLayout,
    vs: &ShaderModule,
    fs: &ShaderModule,
    ty: PipelineType,
    samples: MSAASetting,
) -> RenderPipeline {
    let blend = if ty == PipelineType::Alpha {
        BlendDescriptor {
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        }
    } else {
        BlendDescriptor::REPLACE
    };
    device.create_render_pipeline(&RenderPipelineDescriptor {
        layout,
        vertex_stage: ProgrammableStageDescriptor {
            module: vs,
            entry_point: "main",
        },
        fragment_stage: Some(ProgrammableStageDescriptor {
            module: fs,
            entry_point: "main",
        }),
        rasterization_state: Some(RasterizationStateDescriptor {
            front_face: FrontFace::Cw,
            cull_mode: CullMode::Back,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        }),
        primitive_topology: PrimitiveTopology::TriangleList,
        color_states: &[ColorStateDescriptor {
            format: TextureFormat::Bgra8UnormSrgb,
            color_blend: blend.clone(),
            alpha_blend: blend,
            write_mask: ColorWrite::ALL,
        }],
        depth_stencil_state: Some(DepthStencilStateDescriptor {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: CompareFunction::LessEqual,
            stencil_front: StencilStateFaceDescriptor::IGNORE,
            stencil_back: StencilStateFaceDescriptor::IGNORE,
            stencil_read_mask: 0,
            stencil_write_mask: 0,
        }),
        index_format: IndexFormat::Uint32,
        vertex_buffers: &[VertexBufferDescriptor {
            stride: size_of::<Vertex>() as BufferAddress,
            step_mode: InputStepMode::Vertex,
            attributes: &vertex_attr_array![0 => Float3, 1 => Float3, 2 => Float4, 3 => Float2],
        }],
        sample_count: samples as u32,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    })
}

fn create_depth_buffer(device: &Device, size: &PhysicalSize<u32>, samples: MSAASetting) -> TextureView {
    let depth_texture = device.create_texture(&TextureDescriptor {
        size: Extent3d {
            width: size.width,
            height: size.height,
            depth: 1,
        },
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: samples as u32,
        dimension: TextureDimension::D2,
        format: TextureFormat::Depth32Float,
        usage: TextureUsage::OUTPUT_ATTACHMENT,
    });
    depth_texture.create_default_view()
}

fn create_framebuffer(device: &Device, size: &PhysicalSize<u32>, samples: MSAASetting) -> TextureView {
    let extent = Extent3d {
        width: size.width,
        height: size.height,
        depth: 1,
    };

    let tex = device.create_texture(&TextureDescriptor {
        size: extent,
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: samples as u32,
        dimension: TextureDimension::D2,
        format: TextureFormat::Bgra8UnormSrgb,
        usage: TextureUsage::OUTPUT_ATTACHMENT,
    });
    tex.create_default_view()
}

impl Renderer {
    pub async fn new(window: &Window, samples: MSAASetting) -> Self {
        let screen_size = window.inner_size();

        let surface = Surface::create(window);

        let adapter = Adapter::request(
            &RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
            },
            BackendBit::VULKAN | BackendBit::METAL,
        )
        .await
        .unwrap();

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                extensions: Extensions {
                    anisotropic_filtering: true,
                },
                limits: Limits::default(),
            })
            .await;

        let swapchain_descriptor = SwapChainDescriptor {
            usage: TextureUsage::OUTPUT_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: screen_size.width,
            height: screen_size.height,
            present_mode: PresentMode::Mailbox,
        };
        let swapchain = device.create_swap_chain(&surface, &swapchain_descriptor);

        let vs = include_shader!(vert "test");
        let vs_module = device.create_shader_module(&read_spirv(io::Cursor::new(&vs[..])).unwrap());

        let fs = include_shader!(frag "test");
        let fs_module = device.create_shader_module(&read_spirv(io::Cursor::new(&fs[..])).unwrap());

        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: None,
        });

        let framebuffer = create_framebuffer(&device, &screen_size, samples);
        let depth_buffer = create_depth_buffer(&device, &screen_size, samples);

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            bindings: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                    ty: BindingType::UniformBuffer { dynamic: false },
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        multisampled: false,
                        component_type: TextureComponentType::Uint,
                        dimension: TextureViewDimension::D2,
                    },
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler { comparison: false },
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        let opaque_pipeline = create_pipeline(
            &device,
            &pipeline_layout,
            &vs_module,
            &fs_module,
            PipelineType::Normal,
            samples,
        );
        let alpha_pipeline = create_pipeline(
            &device,
            &pipeline_layout,
            &vs_module,
            &fs_module,
            PipelineType::Alpha,
            samples,
        );

        let mip_creator = MipmapCompute::new(&device);

        // We need to do a couple operations on the whole pile first
        let mut renderer = Self {
            objects: IndexMap::new(),
            object_handle_count: 0,

            textures: IndexMap::new(),
            texture_handle_count: 0,

            camera: Camera {
                location: Vector3::new(-6.0, 0.0, 0.0),
                pitch: 0.0,
                yaw: 0.0,
            },
            screen_size,
            samples,

            surface,
            device,
            queue,
            swapchain,
            framebuffer,
            depth_buffer,
            opaque_pipeline,
            alpha_pipeline,
            pipeline_layout,
            bind_group_layout,
            sampler,

            vert_shader: vs_module,
            frag_shader: fs_module,

            mip_creator,

            command_buffers: Vec::new(),
        };

        // Default texture is texture handle zero, immediately discard the handle, never to be seen again
        renderer.add_texture(&RgbaImage::from_raw(1, 1, vec![0xff, 0xff, 0xff, 0xff]).unwrap());

        renderer
    }

    pub fn add_object(
        &mut self,
        location: Vector3<f32>,
        mesh_verts: Vec<MeshVertex>,
        indices: &[impl ToPrimitive],
    ) -> ObjectHandle {
        self.add_object_texture(location, mesh_verts, indices, &TextureHandle(0))
    }

    pub fn add_object_texture(
        &mut self,
        location: Vector3<f32>,
        mesh_verts: Vec<MeshVertex>,
        indices: &[impl ToPrimitive],
        TextureHandle(tex_idx): &TextureHandle,
    ) -> ObjectHandle {
        let mesh_transparent = is_mesh_transparent(&mesh_verts);

        let tex: &Texture = &self.textures[tex_idx];
        let tex_transparent = tex.transparent;
        let transparent = tex_transparent | mesh_transparent;

        let indices = indices
            .iter()
            .map(|i| i.to_u32().expect("Index too large (>2^32)"))
            .collect_vec();
        let (vertices, indices) = convert_mesh_verts_to_verts(mesh_verts, indices);

        let vertex_buffer = self
            .device
            .create_buffer_with_data(vertices.as_bytes(), BufferUsage::VERTEX);
        let index_buffer = self
            .device
            .create_buffer_with_data(indices.as_bytes(), BufferUsage::INDEX);

        let matrix = generate_matrix(&self.camera.compute_matrix(), location, 800.0 / 600.0);
        let matrix_ref: &[f32; 16] = matrix.as_ref();
        let uniforms = Uniforms {
            _matrix: matrix_ref.clone(),
            _transparent: transparent as u32,
        };
        let uniform_buffer = self
            .device
            .create_buffer_with_data(uniforms.as_bytes(), BufferUsage::UNIFORM | BufferUsage::MAP_WRITE);

        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            layout: &self.bind_group_layout,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::Buffer {
                        buffer: &uniform_buffer,
                        range: 0..64,
                    },
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::TextureView(&tex.texture_view),
                },
                Binding {
                    binding: 2,
                    resource: BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        let mesh_center_offset = find_mesh_center(&vertices);

        let handle = self.object_handle_count;
        self.object_handle_count += 1;
        self.objects.insert(handle, Object {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            texture: 0,
            bind_group,
            uniform_buffer,
            location,
            mesh_center_offset,
            camera_distance: 0.0, // calculated later
            transparent,
            mesh_transparent,
        });
        ObjectHandle(handle)
    }

    pub fn add_texture(&mut self, image: &RgbaImage) -> TextureHandle {
        let transparent = is_texture_transparent(image);

        let extent = Extent3d {
            width: image.width(),
            height: image.height(),
            depth: 1,
        };
        let mip_levels = mip_levels(Vector2::new(image.width(), image.height()));
        let texture = self.device.create_texture(&TextureDescriptor {
            size: extent,
            array_layer_count: 1,
            mip_level_count: mip_levels,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Uint,
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST | TextureUsage::STORAGE,
        });
        let texture_view = texture.create_default_view();
        let tmp_buf = self
            .device
            .create_buffer_with_data(image.as_ref(), BufferUsage::COPY_SRC);
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { todo: 0 });
        encoder.copy_buffer_to_texture(
            BufferCopyView {
                buffer: &tmp_buf,
                offset: 0,
                bytes_per_row: 4 * image.width(),
                rows_per_image: 0,
            },
            TextureCopyView {
                texture: &texture,
                mip_level: 0,
                array_layer: 0,
                origin: Origin3d::ZERO,
            },
            extent,
        );

        self.command_buffers.push(encoder.finish());

        let mip_command = self.mip_creator.compute_mipmaps(
            &self.device,
            &texture,
            Vector2::new(image.width(), image.height()),
            transparent,
        );
        self.command_buffers.extend(mip_command);

        let handle = self.texture_handle_count;
        self.texture_handle_count += 1;

        self.textures.insert(handle, Texture {
            texture_view,
            transparent,
        });
        TextureHandle(handle)
    }

    pub fn get_default_texture() -> TextureHandle {
        TextureHandle(0)
    }

    pub fn set_location(&mut self, ObjectHandle(handle): &ObjectHandle, location: Vector3<f32>) {
        let object: &mut Object = &mut self.objects[handle];

        object.location = location;
    }

    pub fn set_texture(&mut self, ObjectHandle(obj_idx): &ObjectHandle, TextureHandle(tex_idx): &TextureHandle) {
        let obj: &mut Object = &mut self.objects[obj_idx];
        let tex: &Texture = &self.textures[tex_idx];

        obj.texture = *tex_idx;
        obj.transparent = obj.mesh_transparent | tex.transparent;

        obj.bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            layout: &self.bind_group_layout,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::Buffer {
                        buffer: &obj.uniform_buffer,
                        range: 0..64,
                    },
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::TextureView(&tex.texture_view),
                },
                Binding {
                    binding: 2,
                    resource: BindingResource::Sampler(&self.sampler),
                },
            ],
        });
    }

    pub fn set_camera(&mut self, pitch: f32, yaw: f32) {
        self.camera.pitch = pitch;
        self.camera.yaw = yaw;
    }

    pub fn set_camera_location(&mut self, location: Vector3<f32>) {
        self.camera.location = location;
    }

    pub fn resize(&mut self, screen_size: PhysicalSize<u32>, samples: MSAASetting) {
        self.framebuffer = create_framebuffer(&self.device, &screen_size, samples);
        self.depth_buffer = create_depth_buffer(&self.device, &screen_size, samples);
        self.opaque_pipeline = create_pipeline(
            &self.device,
            &self.pipeline_layout,
            &self.vert_shader,
            &self.frag_shader,
            PipelineType::Normal,
            samples,
        );
        self.alpha_pipeline = create_pipeline(
            &self.device,
            &self.pipeline_layout,
            &self.vert_shader,
            &self.frag_shader,
            PipelineType::Alpha,
            samples,
        );
        self.screen_size = screen_size;
        self.samples = samples;

        self.swapchain = self.device.create_swap_chain(&self.surface, &SwapChainDescriptor {
            usage: TextureUsage::OUTPUT_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: screen_size.width,
            height: screen_size.height,
            present_mode: PresentMode::Mailbox,
        });
    }

    pub fn get_samples(&self) -> MSAASetting {
        self.samples
    }

    pub fn set_samples(&mut self, samples: MSAASetting) {
        self.resize(self.screen_size, samples);
    }

    async fn recompute_uniforms(&mut self) {
        let camera_mat = self.camera.compute_matrix();
        for object in self.objects.values() {
            let mut buf = object
                .uniform_buffer
                .map_write(0, size_of::<Uniforms>() as u64)
                .await
                .expect("Could not map buffer");
            let matrix = generate_matrix(
                &camera_mat,
                object.location,
                self.screen_size.width as f32 / self.screen_size.height as f32,
            );
            let matrix_ref: &[f32; 16] = matrix.as_ref();
            let uniforms = Uniforms {
                _matrix: matrix_ref.clone(),
                _transparent: object.transparent as u32,
            };
            buf.as_slice().copy_from_slice(uniforms.as_bytes());
        }
    }

    fn compute_object_distances(&mut self) {
        for obj in self.objects.values_mut() {
            let mesh_center: Vector3<f32> = obj.location + obj.mesh_center_offset;
            let camera_mesh_vector: Vector3<f32> = self.camera.location - mesh_center;
            let distance = camera_mesh_vector.magnitude2();
            obj.camera_distance = distance;
            // println!(
            //     "{} - {} {} {}",
            //     obj.camera_distance, obj.transparent, obj.mesh_transparent, self.textures[&obj.texture].transparent
            // );
        }
    }

    fn sort_objects(&mut self) {
        self.objects.sort_by(|_, lhs: &Object, _, rhs: &Object| {
            lhs.transparent.cmp(&rhs.transparent).then_with(|| {
                if lhs.transparent {
                    // we can only get here if they are both of the same transparency,
                    // so I can use the transparency for one as the transparency for both
                    rhs.camera_distance
                        .partial_cmp(&lhs.camera_distance)
                        .unwrap_or(Ordering::Equal)
                } else {
                    lhs.camera_distance
                        .partial_cmp(&rhs.camera_distance)
                        .unwrap_or(Ordering::Equal)
                }
            })
        });
    }

    pub async fn render(&mut self) {
        self.recompute_uniforms().await;
        self.compute_object_distances();
        self.sort_objects();

        let frame = self.swapchain.get_next_texture().unwrap();

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { todo: 0 });

        {
            let (attachment, resolve_target) = if self.samples == MSAASetting::X1 {
                (&frame.view, None)
            } else {
                (&self.framebuffer, Some(&frame.view))
            };
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment,
                    resolve_target,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_color: Color {
                        r: 0.3,
                        g: 0.3,
                        b: 0.3,
                        a: 1.0,
                    },
                }],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_buffer,
                    depth_load_op: LoadOp::Clear,
                    depth_store_op: StoreOp::Store,
                    stencil_load_op: LoadOp::Clear,
                    stencil_store_op: StoreOp::Store,
                    clear_depth: 1.0,
                    clear_stencil: 0,
                }),
            });
            let mut opaque_ended = false;
            rpass.set_pipeline(&self.opaque_pipeline);
            for object in self.objects.values() {
                if object.transparent && !opaque_ended {
                    rpass.set_pipeline(&self.alpha_pipeline);
                    opaque_ended = true;
                }

                rpass.set_bind_group(0, &object.bind_group, &[]);
                rpass.set_vertex_buffer(0, &object.vertex_buffer, 0, 0);
                rpass.set_index_buffer(&object.index_buffer, 0, 0);
                rpass.draw_indexed(0..(object.index_count as u32), 0, 0..1);
            }
        }

        self.command_buffers.push(encoder.finish());

        self.queue.submit(&self.command_buffers);
        self.command_buffers.clear();
    }
}
