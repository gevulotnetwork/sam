import "assert" as assert;

describe("Builtin Functions", || {
    it("should be able to log messages", || {
        log("This is a log message");
        assert(true, "Log function exists");
    });

    it("should be able to execute shell commands", || {
        let result = exec("echo hello");
        assert(result == "hello\n", "Exec function returned unexpected result");
    });

    it("should be able to set environment variables", || {
        set_env("TEST_VAR", "test_value");
        let result = exec("echo $TEST_VAR");
        assert(result == "test_value\n", "Environment variable was not set correctly");
    });

    it("should be able to get environment variables", || {
        set_env("TEST_VAR", "test_value");
        let result = get_env("TEST_VAR");
        assert(result == "test_value", "Environment variable was not set correctly");
    });

    it("should be able to start and stop components", || {
        start_component("test-container");
        stop_component("test-container");
        assert(true, "Component control functions exist");
    });

    it("should support multiple sleep formats", || {
        sleep("100ms");
        sleep("1s");
        assert(true, "Sleep function works with time strings");
    });

    it("should support wait_until with duration string", || {
        let start = timestamp();
        wait_until(|| {
            start.elapsed() > 0.1
        }, "1s");
        assert(true, "Wait until works with duration string");
    });

    it("should support wait_until with milliseconds", || {
        let start = timestamp();
        wait_until(|| {
            start.elapsed() > 0.1
        }, 1000);
        assert(true, "Wait until works with milliseconds");
    });

    it("should be possible to catch errors", || {
        try {
            exec("false");
        } catch (e) {
            assert(true, "Error was caught");
        }
    });

    describe("kv store", || {
        it("should be able to set and get an integer", || {
            set("a", 1);
            assert::eq(get("a"), 1);
        });

        it("should be able to set and get a string", || {
            set("b", "Hello");
            assert::eq(get("b"), "Hello");
        });

        it("should be able to set and get a boolean", || {
            set("c", true);
            assert::eq(get("c"), true);
        });

        it("should be able to set and get a map", || {
            set("d", #{"a": 1, "b": 2});
            assert::eq(get("d"), #{"a": 1, "b": 2});
        });

        it("should be able to set and get a list", || {
            set("e", [1, 2, 3]);
            assert::eq(get("e"), [1, 2, 3]);
        });
    });


    task("Show alternative task syntax", || {
        task("sub-task", || {
            step("assert that substeps work", || {
                assert(true, "Alternative task syntax works");
            });
        });
    });

    describe("spawn", || {
        it("should be possible to spawn a task", || {
            print("");
            let id = spawn_task(|| {
                print("Hello from the spawned task");
            });
            wait_for_task(id);
        });

        it("should be able to spawn multiple tasks", || {
            print("");
            let ids = [];
            for i in 0..10 {
                let i = i; // note that this closure specific clone needs to be done here, like in many other languages
                ids.push(spawn_task(|| {
                    let delay = random_int(100, 500);
                    sleep(`${delay}ms`);
                    print(`Hello from the spawned task ${i} after ${delay}ms`);
                }));
            }
            print("all spawned");
            print("waiting for tasks");
            wait_for_tasks(ids);
        });
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
