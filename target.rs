#![cfg(feature = "llvm")]

use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::OptimizationLevel;
use std::path::Path;

/// Prepares the LLVM targets and writes the IR Module to a native Object file.
pub fn emit_native_executable(module: &Module, output_path: &Path) -> Result<(), String> {
    // 1. Initialize native host targets (Windows/Mac/Linux)
    Target::initialize_native(&InitializationConfig::default())
        .map_err(|e| format!("Failed to initialize native target: {}", e))?;

    let target_triple = TargetMachine::get_default_triple();
    
    let target = Target::from_triple(&target_triple)
        .map_err(|e| format!("Target error: {}", e))?;

    // 2. Create the Target Machine (Optimized for O3)
    let target_machine = target
        .create_target_machine(
            &target_triple,
            "generic",
            "",
            OptimizationLevel::Aggressive,
            RelocMode::Default,
            CodeModel::Default,
        )
        .ok_or("Could not create TargetMachine")?;

    module.set_data_layout(&target_machine.get_target_data().get_data_layout());
    module.set_triple(&target_triple);

    // 3. Run Optimization Passes (Constant Folding, Dead Code Elimination, etc.)
    optimize_module(module);

    // 4. Emit Object File (.o / .obj)
    target_machine
        .write_to_file(module, FileType::Object, output_path)
        .map_err(|e| format!("Failed to write object file: {}", e))?;

    Ok(())
}

/// Runs standard LLVM optimization passes over the generated IR.
fn optimize_module(module: &Module) {
    let pass_manager = PassManager::create(());
    
    pass_manager.add_instruction_combining_pass();
    pass_manager.add_reassociate_pass();
    pass_manager.add_cfg_simplification_pass();
    pass_manager.add_basic_alias_analysis_pass();
    pass_manager.add_promote_memory_to_register_pass(); // Highly important: turns allocas into fast registers!
    pass_manager.add_instruction_simplify_pass();
    pass_manager.add_dead_arg_elimination_pass();
    
    pass_manager.run_on(module);
}