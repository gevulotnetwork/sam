import "module" as mod;

describe("Example Webserver", || {
    it("should be possible to start a component on demand", || {
        start_component("caddy");
        require(true, "Component did not start");
    });

    it("serves files", || {
        let response = exec("curl -s http://localhost:8080/index.html");
        require(response == "Hello, World!", "Caddy did not serve the expected file");
    });

    it("can also serve 100 files", || {
        for i in 0..100 {
            let response = exec("curl -s http://localhost:8080/index.html");
            require(response == "Hello, World!", "Caddy did not serve the expected file");
        }
    });
});

describe("Injected Functions", || {
    it("should be able to log messages", || {
        log("This is a log message");
        require(true, "Log function exists");
    });

    it("should be able to execute shell commands", || {
        let result = exec("echo hello");
        require(result == "hello\n", "Exec function returned unexpected result");
    });

    it("should be able to set environment variables", || {
        set_env("TEST_VAR", "test_value");
        let result = exec("echo $TEST_VAR");
        require(result == "test_value\n", "Environment variable was not set correctly");
    });

    it("should be able to start and stop components", || {
        start_component("caddy");
        stop_component("caddy"); 
        require(true, "Component control functions exist");
    });

    it("should support multiple sleep formats", || {
        sleep("100ms");
        sleep("1s");
        require(true, "Sleep function works with time strings");
    });

    it("should support wait_until with duration string", || {
        let start = timestamp();
        wait_until(|| {
            start.elapsed() > 0.1
        }, "1s");
        require(true, "Wait until works with duration string");
    });

    it("should support wait_until with milliseconds", || {
        let start = timestamp();
        wait_until(|| {
            start.elapsed() > 0.1
        }, 1000);
        require(true, "Wait until works with milliseconds");
    });
});

