use crate::bindings;

use super::{
    object::{AsObject, Object},
    pipeline_resource_signature::PipelineResourceSignature,
    resource_mapping::ResourceMapping,
    shader_resource_variable::ShaderResourceVariable,
};

pub struct ShaderResourceBinding {
    pub(crate) m_shader_resource_binding: *mut bindings::IShaderResourceBinding,
    m_virtual_functions: *mut bindings::IShaderResourceBindingVtbl,

    m_object: Object,
}

impl AsObject for ShaderResourceBinding {
    fn as_object(&self) -> &Object {
        &self.m_object
    }
}

impl ShaderResourceBinding {
    pub(crate) fn new(srb_ptr: *mut bindings::IShaderResourceBinding) -> Self {
        ShaderResourceBinding {
            m_shader_resource_binding: srb_ptr,
            m_virtual_functions: unsafe { (*srb_ptr).pVtbl },
            m_object: Object::new(srb_ptr as *mut bindings::IObject),
        }
    }

    fn get_pipeline_resource_signature(&self) -> Option<&PipelineResourceSignature> {
        todo!()
    }

    fn bind_resources(
        &mut self,
        shader_stages: bindings::SHADER_TYPE,
        resource_mapping: &ResourceMapping,
        flags: bindings::BIND_SHADER_RESOURCES_FLAGS,
    ) {
        unsafe {
            (*self.m_virtual_functions)
                .ShaderResourceBinding
                .BindResources
                .unwrap_unchecked()(
                self.m_shader_resource_binding,
                shader_stages,
                resource_mapping.m_resource_mapping,
                flags,
            )
        }
    }

    fn check_resources(
        &self,
        shader_stages: bindings::SHADER_TYPE,
        resource_mapping: &ResourceMapping,
        flags: bindings::BIND_SHADER_RESOURCES_FLAGS,
    ) -> bindings::SHADER_RESOURCE_VARIABLE_TYPE_FLAGS {
        unsafe {
            (*self.m_virtual_functions)
                .ShaderResourceBinding
                .CheckResources
                .unwrap_unchecked()(
                self.m_shader_resource_binding,
                shader_stages,
                resource_mapping.m_resource_mapping,
                flags,
            )
        }
    }

    fn get_variables(
        &self,
        shader_type: bindings::SHADER_TYPE,
    ) -> Option<&[ShaderResourceVariable]> {
        todo!()
    }

    fn static_resources_initialized(&self) -> bool {
        unsafe {
            (*self.m_virtual_functions)
                .ShaderResourceBinding
                .StaticResourcesInitialized
                .unwrap_unchecked()(self.m_shader_resource_binding)
        }
    }
}
