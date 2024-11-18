describe("Builtin Functions", || {
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
        start_component("test-container");
        stop_component("test-container");
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

    it("should be possible to catch errors", || {
        try {
            exec("false");
        } catch (e) {
            require(true, "Error was caught");
        }
    });

    // it("should be able to assert a failure", || {
    //     assert(false, "This test should fail");
    //     assert(true, "This test should pass");
    //     assert(false, "This other test should fail as well");
    // });

    // it("should be able to require a failure", || {
    //     assert(false, "This assertion should fail");
    //     require(false, "This requirement should fail");
    //     assert(false, "This assertion should not appear in the output, the test is over");
    // });
});
