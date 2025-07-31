use rustyscript::{json_args, serde_json, Module, RuntimeBuilder};

fn main() -> Result<(), rustyscript::Error> {
    println!("Testing HTTPS fetch with rustyscript...");

    // Create a runtime with default web extensions enabled
    let mut runtime = RuntimeBuilder::new().build()?;

    // JavaScript code that fetches from example.com via HTTPS
    let module = Module::new(
        "https_fetch_test.js",
        r#"
        export default async function testHttpsFetch() {
            try {
                console.log("Making HTTPS request to httpbin.org...");
                const response = await fetch('https://httpbin.org/json');
                
                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
                }
                
                const text = await response.text();
                console.log(`Success! Response status: ${response.status}`);
                console.log(`Content length: ${text.length} characters`);
                
                // Try to parse as JSON for more interesting output
                let jsonData;
                try {
                    jsonData = JSON.parse(text);
                } catch {
                    jsonData = { raw: text };
                }
                
                return {
                    success: true,
                    status: response.status, 
                    statusText: response.statusText,
                    contentLength: text.length,
                    data: jsonData,
                    headers: Object.fromEntries(response.headers.entries())
                };
            } catch (error) {
                console.error(`HTTPS fetch failed: ${error.message}`);
                return {
                    success: false,
                    error: error.message
                };
            }
        }
        "#,
    );

    // Load and execute the module
    let handle = runtime.load_module(&module)?;
    let result: serde_json::Value = runtime.call_entrypoint(&handle, json_args!())?;

    // Print the results in a nice format
    println!("\n Test Results:");
    println!("{}", serde_json::to_string_pretty(&result)?);

    // Verify the test was successful
    if let Some(success) = result.get("success").and_then(|v| v.as_bool()) {
        if success {
            println!("\n HTTPS fetch test completed successfully!");
            println!("The rustls crypto provider has been properly initialized.");
        } else {
            println!("\n HTTPS fetch test failed!");
            if let Some(error) = result.get("error") {
                println!("Error: {}", error);
            }
        }
    }

    Ok(())
}
