use super::ExtensionTrait;
use deno_core::{extension, op2, Extension, OpState};
use std::rc::Rc;

/// A structure to store exit code in OpState when script exit is requested
#[derive(Clone, Debug)]
pub struct ScriptExitRequest {
    pub code: i32,
}

/// Wrapper for V8 isolate handle that can be stored in OpState
#[derive(Clone)]
pub struct V8IsolateHandle(pub Rc<deno_core::v8::IsolateHandle>);

/// Request script termination with the given exit code (replaces dangerous std::process::exit)
/// This terminates V8 execution immediately for zero-tolerance termination
#[op2(fast)]
fn op_script_exit(state: &mut OpState, #[smi] code: i32) -> Result<(), crate::Error> {
    // Store the exit request in OpState for retrieval after termination
    let exit_request = ScriptExitRequest { code };
    state.put(exit_request);

    // IMMEDIATE TERMINATION: Terminate V8 execution immediately
    // This will stop ANY JavaScript execution, including infinite loops
    if let Some(isolate_handle) = state.try_borrow::<V8IsolateHandle>() {
        isolate_handle.0.terminate_execution();
    }

    // Return Ok - the V8 termination will handle immediate stopping
    Ok(())
}

extension!(
    init_os,
    deps = [rustyscript],
    ops = [op_script_exit],
    esm_entry_point = "ext:init_os/init_os.js",
    esm = [ dir "src/ext/os", "init_os.js" ],
);

impl ExtensionTrait<()> for init_os {
    fn init((): ()) -> Extension {
        init_os::init()
    }
}

pub fn extensions(is_snapshot: bool) -> Vec<Extension> {
    vec![init_os::build((), is_snapshot)]
}

#[cfg(test)]
mod test;
