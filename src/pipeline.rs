use crate::{Device, Error};
use std::{fs::File, io::Read, ops::Range, path::Path};

pub use d3d12::PipelineState as Pipeline;
pub use d3d12::RootSignature as PipelineLayout;

pub struct Shader {
    data: Vec<u8>,
}

impl Shader {
    pub fn new(name: &str, source: &str, entry_point: &str, target: &str) -> Result<Self, Error> {
        match hassle_rs::compile_hlsl(name, source, entry_point, target, &["/Zi"], &[])
            .and_then(|shader| hassle_rs::validate_dxil(&shader))
        {
            Ok(data) => Ok(Shader { data }),
            Err(err) => Err(Error::Shader { cause: err }),
        }
    }

    pub fn new_from_path<P: AsRef<Path>>(
        name: &str,
        path: P,
        entry_point: &str,
        target: &str,
    ) -> Result<Self, Error> {
        let mut shader_file = File::open(path)?;
        let mut shader_source = String::new();
        shader_file.read_to_string(&mut shader_source)?;

        Self::new(name, &shader_source, entry_point, target)
    }

    pub(crate) fn bytecode(&self) -> d3d12::Shader {
        d3d12::Shader::from_raw(&self.data)
    }
}

pub enum LayoutDesc {
    Constant { space: u32, binding: u32, num: u32 },
    Descriptors(Vec<BindingDesc>),
}

pub use d3d12::DescriptorRangeType as DescriptorTy;
pub struct BindingDesc {
    pub ty: DescriptorTy,
    pub space: u32,
    pub bindings: Range<u32>,
}

impl Device {
    pub fn create_pipeline_layout(&self, descs: &[LayoutDesc]) -> Result<PipelineLayout, Error> {
        let num_ranges = descs
            .iter()
            .map(|desc| match desc {
                LayoutDesc::Constant { .. } => 0,
                LayoutDesc::Descriptors(bindings) => bindings.len(),
            })
            .sum();

        let mut last_range = 0;
        let mut descriptor_ranges = Vec::with_capacity(num_ranges);
        let mut parameters = Vec::new();

        for desc in descs {
            match desc {
                LayoutDesc::Descriptors(ref bindings) => {
                    for binding in bindings {
                        descriptor_ranges.push(d3d12::DescriptorRange::new(
                            binding.ty,
                            binding.bindings.end - binding.bindings.start,
                            d3d12::Binding {
                                register: binding.bindings.start,
                                space: binding.space,
                            },
                            !0, // append
                        ));
                    }
                    let cur_range = descriptor_ranges.len();
                    parameters.push(d3d12::RootParameter::descriptor_table(
                        d3d12::ShaderVisibility::All,
                        &descriptor_ranges[last_range..cur_range],
                    ));
                    last_range = cur_range;
                }
                LayoutDesc::Constant {
                    space,
                    binding,
                    num,
                } => {
                    parameters.push(d3d12::RootParameter::constants(
                        d3d12::ShaderVisibility::All,
                        d3d12::Binding {
                            register: *binding,
                            space: *space,
                        },
                        *num,
                    ));
                }
            }
        }

        let ((signature, _), _) = d3d12::RootSignature::serialize(
            d3d12::RootSignatureVersion::V1_0,
            &parameters,
            &[], // TODO
            d3d12::RootSignatureFlags::empty(),
        );
        let (layout, _) = self.create_root_signature(signature, 0);
        Ok(layout)
    }
}
