const core = globalThis.Deno.core;

function exit(code = 0, reason) {
    if (typeof code !== "number" || !Number.isInteger(code) || code < 0) {
        throw new TypeError("Exit code must be a non-negative integer");
    }

    // Dispatch unload event before exit (similar to browser/Deno behavior)
    if (typeof globalThis.dispatchEvent === "function" && typeof Event !== "undefined") {
        try {
            globalThis.dispatchEvent(new Event("unload"));
        } catch (e) {
            // Ignore errors in unload event dispatch
        }
    }

    // Call the script exit operation - this terminates V8 execution immediately
    // No JavaScript code can execute after this call due to immediate termination
    core.ops.op_script_exit(code);

    // This line will NEVER execute due to immediate exception from the operation above
    throw new Error("Script execution should have been terminated immediately");
}

// Make exit available on the global Deno object
if (typeof globalThis.Deno === "undefined") {
    globalThis.Deno = {};
}

globalThis.Deno.exit = exit;

export { exit };
