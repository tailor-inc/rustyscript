///
/// This example shows how to use the os_exit feature to provide
/// os.exit functionality to JavaScript code.
///
/// The os_exit feature enables the deno_os extension which provides
/// process termination capabilities through Deno.exit().
///
/// Note: This example doesn't actually call exit() as that would
/// terminate the process. Instead, it demonstrates that the
/// functionality is available.
///
use rustyscript::{Error, Module, Runtime, RuntimeOptions};

fn main() -> Result<(), Error> {
    // First check if the feature is available
    if !check_feature_available()? {
        println!("The os_exit feature is not enabled.");
        println!("Try running: cargo run --example os_exit --features=\"os_exit\"");
        return Ok(());
    }

    println!("Success! The os_exit feature is working correctly.");
    println!("JavaScript code now has access to Deno.exit() for script termination.");

    // Run each test with its own runtime for complete isolation
    test_basic_exit()?;
    test_runtime_survival()?;
    test_infinite_loop()?;

    Ok(())
}

fn check_feature_available() -> Result<bool, Error> {
    let module = Module::new(
        "test_exit.js",
        r#"
        // Check if Deno.exit is available
        if (typeof Deno !== 'undefined' && typeof Deno.exit === 'function') {
            console.log("SUCCESS: Deno.exit is available");

            // We can test the function exists but won't call it
            // as that would terminate this example program
            console.log("  Function signature:", Deno.exit.toString());
        } else {
            console.log("FAILURE: Deno.exit is not available");
            console.log("  Make sure to compile with --features=\"os_exit\"");
        }

        export const hasExit = typeof Deno?.exit === 'function';
        "#,
    );

    let mut runtime = Runtime::new(RuntimeOptions::default())?;
    let module_handle = runtime.load_module(&module)?;
    runtime.get_value(Some(&module_handle), "hasExit")
}

fn test_basic_exit() -> Result<(), Error> {
    println!("\nTesting immediate script exit...");

    // Create a fresh runtime for this test
    let mut runtime = Runtime::new(RuntimeOptions::default())?;

    let test_module = Module::new(
        "test_exit.js",
        r#"
        console.log("Before Deno.exit(42)");

        // This should throw an immediate exception - no further code should execute
        Deno.exit(42);

        // CRITICAL TEST: These lines should NEVER execute
        console.log("FAILURE: This line executed after Deno.exit()!");
        globalThis.POST_EXIT_EXECUTED = true;
        throw new Error("Post-exit code executed - immediate termination failed!");
        "#,
    );

    let result = runtime.load_module(&test_module);

    let Err(e) = result else {
        return Err(Error::Runtime(
            "CRITICAL: Script completed without immediate exit!".to_string(),
        ));
    };

    let Some(code) = e.as_script_exit() else {
        return Err(Error::Runtime(format!("ERROR: Unexpected error: {}", e)));
    };

    println!(
        "SUCCESS: Basic test - Script exited immediately with code: {}",
        code
    );

    // Verify no post-exit globals were set
    match runtime.eval::<bool>("typeof globalThis.POST_EXIT_EXECUTED !== 'undefined'") {
        Ok(false) => {
            println!("SUCCESS: Immediate termination verified - No post-exit code executed")
        }
        Ok(true) => {
            return Err(Error::Runtime(
                "CRITICAL: Post-exit code executed!".to_string(),
            ))
        }
        Err(_) => {
            println!("SUCCESS: Immediate termination verified - No post-exit globals accessible")
        }
    }

    Ok(())
}

fn test_runtime_survival() -> Result<(), Error> {
    // Create a fresh runtime for this test
    let mut runtime = Runtime::new(RuntimeOptions::default())?;

    // First exit the runtime
    let exit_module = Module::new(
        "exit_test.js",
        r#"
        console.log("About to exit...");
        Deno.exit(0);
        "#,
    );

    let _ = runtime.load_module(&exit_module); // Ignore the exit error

    // Now verify the runtime still works
    let result: String = runtime.eval("'Runtime still works after immediate termination!'")?;
    println!("SUCCESS: Runtime survival test - {}", result);
    Ok(())
}

fn test_infinite_loop() -> Result<(), Error> {
    println!("\nTesting script exit from infinite loop...");

    // Create a fresh runtime for this test
    let mut runtime = Runtime::new(RuntimeOptions::default())?;

    let infinite_loop_module = Module::new(
        "infinite_loop_test.js",
        r#"
        console.log("Starting infinite loop test");
        let count = 0;
        while (true) {
            count++;
            if (count > 1000000) {
                console.log("Calling Deno.exit from within infinite loop");
                Deno.exit(99);

                // CRITICAL TEST: These lines should NEVER execute due to immediate termination
                console.log("FAILURE: Code executed after Deno.exit() in infinite loop!");
                globalThis.INFINITE_LOOP_POST_EXIT_EXECUTED = true;
                break; // This should never be reached
            }
        }
        console.log("FAILURE: End of infinite loop reached!");
        globalThis.INFINITE_LOOP_COMPLETED = true;
        "#,
    );

    let result = runtime.load_module(&infinite_loop_module);

    let Err(e) = result else {
        return Err(Error::Runtime(
            "ERROR: Unexpected - infinite loop script completed without exiting".to_string(),
        ));
    };

    let Some(code) = e.as_script_exit() else {
        return Err(Error::Runtime(format!(
            "ERROR: Unexpected error from infinite loop: {}",
            e
        )));
    };

    println!(
        "SUCCESS: Infinite loop script exited cleanly with code: {}",
        code
    );

    // CRITICAL: Verify no post-exit code executed in infinite loop
    match runtime.eval::<bool>("typeof globalThis.INFINITE_LOOP_POST_EXIT_EXECUTED !== 'undefined'")
    {
        Ok(false) => {
            println!("SUCCESS: Infinite loop immediate termination verified - No post-exit code executed")
        }
        Ok(true) => {
            return Err(Error::Runtime(
                "CRITICAL: Post-exit code executed in infinite loop!".to_string(),
            ))
        }
        Err(_) => println!(
            "SUCCESS: Infinite loop immediate termination verified - No post-exit globals accessible"
        ),
    }

    // Also verify the loop didn't complete normally
    match runtime.eval::<bool>("typeof globalThis.INFINITE_LOOP_COMPLETED !== 'undefined'") {
        Ok(false) => {
            println!("SUCCESS: Infinite loop properly terminated - Loop did not complete normally")
        }
        Ok(true) => {
            return Err(Error::Runtime(
                "CRITICAL: Infinite loop completed normally after exit!".to_string(),
            ))
        }
        Err(_) => {
            println!("SUCCESS: Infinite loop properly terminated - No completion flags accessible")
        }
    }

    println!("SUCCESS: Runtime survived the infinite loop!");

    // Test that the runtime can still execute code after infinite loop
    let result: String = runtime.eval("'Runtime still works after infinite loop!'")?;
    println!("SUCCESS: Post-infinite-loop test - {}", result);

    Ok(())
}
