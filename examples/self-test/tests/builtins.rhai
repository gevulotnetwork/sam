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

    describe("Encodings", || {
        describe("JSON", || {
            it("should be able to parse JSON", || {
                let json = `{"a": 1, "b": 2}`;
                let parsed = parse_json(json);
                assert::eq(parsed, #{"a": 1, "b": 2});
            });

            it("should be able to parse scalar json values", || {
                let json = `"Hello"`;
                let parsed = parse_json(json);
                assert::eq(parsed, "Hello");
            });

            it("should be possible to encode to json", || {
                let value = #{"a": 1, "b": 2};
                let encoded = to_json(value);
                assert::eq(encoded, `{"a":1,"b":2}`);
            });
        });

        describe("YAML", || {
            it("should be able to parse YAML", || {
                let yaml = `---
                a: 1
                b: 2
                `;
                let parsed = parse_yaml(yaml);
                assert::eq(parsed, #{"a": 1, "b": 2});
            });

            it("should be possible to encode to yaml", || {
                let value = #{"a": 1, "b": 2};
                let encoded = to_yaml(value);
                let expected = "a: 1\nb: 2\n";
                assert::eq(encoded, expected);
            });
        });

        describe("TOML", || {
            it("should be able to parse TOML", || {
                let toml = `
                a = 1
                b = 2
                `;
                let parsed = parse_toml(toml);
                assert::eq(parsed, #{"a": 1, "b": 2});
            });

            it("should be possible to encode to toml", || {
                let value = #{"a": 1, "b": 2};
                let encoded = to_toml(value);
                let expected = "a = 1\nb = 2\n";
                assert::eq(encoded, expected);
            });
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
