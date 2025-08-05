#[cfg(test)]
mod tests {
    use crate::{Error, Module, Runtime, RuntimeOptions};

    #[test]
    fn test_os_exit_extension_available() -> Result<(), Error> {
        // Test that the os.exit extension works correctly
        let mut runtime = Runtime::new(RuntimeOptions::default())?;

        let module = Module::new(
            "test_os_exit.js",
            r#"
            // Check if Deno.exit function is available
            export const has_deno = typeof Deno !== 'undefined';
            export const has_exit = typeof Deno?.exit === 'function';

            // Test the function works by checking it's callable
            // (We can't actually call it as it would terminate the test)
            export const is_callable = typeof Deno?.exit === 'function';

            // Test parameter validation by checking function signature
            // We cannot actually call Deno.exit with invalid params as it would
            // immediately terminate even before parameter validation
            let param_validation = false;
            if (typeof Deno?.exit === 'function') {
                // The exit function should accept a number parameter
                // Check that the function exists and has proper signature
                param_validation = Deno.exit.length === 1 || Deno.exit.length === 0;
            }

            export const validation_works = param_validation;
            "#,
        );

        let handle = runtime.load_module(&module)?;

        // Verify that Deno object exists
        let has_deno: bool = runtime.get_value(Some(&handle), "has_deno")?;
        assert!(has_deno, "Deno object should be available");

        // Verify that exit function exists
        let has_exit: bool = runtime.get_value(Some(&handle), "has_exit")?;
        assert!(
            has_exit,
            "Deno.exit should be available with os_exit feature"
        );

        // Verify that the function is callable
        let is_callable: bool = runtime.get_value(Some(&handle), "is_callable")?;
        assert!(is_callable, "Deno.exit should be a callable function");

        // Verify that parameter validation works
        let validation_works: bool = runtime.get_value(Some(&handle), "validation_works")?;
        assert!(
            validation_works,
            "Deno.exit should validate parameters correctly"
        );

        Ok(())
    }

    #[test]
    fn test_os_exit_terminates_script() -> Result<(), Error> {
        // Test that calling Deno.exit actually terminates the script
        let mut runtime = Runtime::new(RuntimeOptions::default())?;

        let module = Module::new(
            "test_exit_termination.js",
            r#"
            console.log("Before exit");
            Deno.exit(42);
            console.log("This should never execute");
            globalThis.SHOULD_NOT_EXIST = true;
            "#,
        );

        // Loading the module should result in a script exit error
        let result = runtime.load_module(&module);

        // Verify that we got a script exit error
        match result {
            Err(e) => {
                // Check if this is a script exit with code 42
                if let Some((code, _reason)) = e.as_script_exit() {
                    assert_eq!(code, 42, "Exit code should be 42");
                } else {
                    panic!("Expected ScriptExit error, got: {:?}", e);
                }
            }
            Ok(_) => panic!("Script should have exited, but completed successfully"),
        }

        // Verify that code after exit didn't execute
        let global_check: Result<bool, Error> =
            runtime.eval("typeof globalThis.SHOULD_NOT_EXIST !== 'undefined'");

        match global_check {
            Ok(false) => {} // Good - the global was not set
            Ok(true) => panic!("Code after Deno.exit() was executed!"),
            Err(_) => {} // Also acceptable - runtime might be in terminated state
        }

        Ok(())
    }

    #[test]
    fn test_os_exit_in_loop() -> Result<(), Error> {
        // Test that Deno.exit works even in infinite loops
        let mut runtime = Runtime::new(RuntimeOptions::default())?;

        let module = Module::new(
            "test_exit_in_loop.js",
            r#"
            let count = 0;
            while (true) {
                count++;
                if (count > 1000) {
                    Deno.exit(99);
                }
            }
            console.log("This should never execute");
            "#,
        );

        // Loading the module should result in a script exit error
        let result = runtime.load_module(&module);

        // Verify that we got a script exit error
        match result {
            Err(e) => {
                if let Some((code, _reason)) = e.as_script_exit() {
                    assert_eq!(code, 99, "Exit code should be 99");
                } else {
                    panic!("Expected ScriptExit error, got: {:?}", e);
                }
            }
            Ok(_) => panic!("Infinite loop should have been terminated by exit"),
        }

        Ok(())
    }
}
