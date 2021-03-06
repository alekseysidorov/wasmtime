use super::create_handle::create_handle;
use crate::externals::{LinearMemory, MemoryCreator};
use crate::Store;
use crate::{Limits, MemoryType};
use anyhow::Result;
use wasmtime_environ::entity::PrimaryMap;
use wasmtime_environ::{wasm, MemoryPlan, Module, WASM_PAGE_SIZE};
use wasmtime_runtime::{
    InstanceHandle, RuntimeLinearMemory, RuntimeMemoryCreator, VMMemoryDefinition,
};

use std::sync::Arc;

pub fn create_handle_with_memory(store: &Store, memory: &MemoryType) -> Result<InstanceHandle> {
    let mut module = Module::new();

    let memory = wasm::Memory {
        minimum: memory.limits().min(),
        maximum: memory.limits().max(),
        shared: false, // TODO
    };
    let tunable = Default::default();

    let memory_plan = wasmtime_environ::MemoryPlan::for_memory(memory, &tunable);
    let memory_id = module.local.memory_plans.push(memory_plan);
    module.exports.insert(
        "memory".to_string(),
        wasmtime_environ::Export::Memory(memory_id),
    );

    create_handle(
        module,
        store,
        PrimaryMap::new(),
        Default::default(),
        Box::new(()),
    )
}

struct LinearMemoryProxy {
    mem: Box<dyn LinearMemory>,
}

impl RuntimeLinearMemory for LinearMemoryProxy {
    fn size(&self) -> u32 {
        self.mem.size()
    }

    fn grow(&self, delta: u32) -> Option<u32> {
        self.mem.grow(delta)
    }

    fn vmmemory(&self) -> VMMemoryDefinition {
        VMMemoryDefinition {
            base: self.mem.as_ptr(),
            current_length: self.mem.size() as usize * WASM_PAGE_SIZE as usize,
        }
    }
}

#[derive(Clone)]
pub(crate) struct MemoryCreatorProxy {
    pub(crate) mem_creator: Arc<dyn MemoryCreator>,
}

impl RuntimeMemoryCreator for MemoryCreatorProxy {
    fn new_memory(&self, plan: &MemoryPlan) -> Result<Box<dyn RuntimeLinearMemory>, String> {
        let ty = MemoryType::new(Limits::new(plan.memory.minimum, plan.memory.maximum));
        self.mem_creator
            .new_memory(ty)
            .map(|mem| Box::new(LinearMemoryProxy { mem }) as Box<dyn RuntimeLinearMemory>)
    }
}
