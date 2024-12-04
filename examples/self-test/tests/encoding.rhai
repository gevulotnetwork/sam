import "assert" as assert;

describe("Encoding Functions", || {
    describe("JSON", || {
        it("should parse basic JSON objects", || {
            let json = `{"a": 1, "b": "hello", "c": true}`;
            let parsed = parse_json(json);
            assert::eq(parsed, #{"a": 1, "b": "hello", "c": true});
        });

        it("should parse JSON arrays", || {
            let json = `[1, "hello", true]`;
            let parsed = parse_json(json);
            assert::eq(parsed, [1, "hello", true]);
        });

        it("should convert objects to JSON", || {
            let obj = #{"a": 1, "b": "hello", "c": true};
            let json = to_json(obj);
            assert::eq(json, `{"a":1,"b":"hello","c":true}`);
        });

        it("should convert objects to pretty JSON", || {
            let obj = #{"a": 1, "b": "hello"};
            let json = to_json_pretty(obj);
            let expected = `{
  "a": 1,
  "b": "hello"
}`;
            assert::eq(json, expected);
        });
    });

    describe("YAML", || {
        it("should parse basic YAML objects", || {
            let yaml = "a: 1\nb: hello\nc: true";
            let parsed = parse_yaml(yaml);
            assert::eq(parsed, #{"a": 1, "b": "hello", "c": true});
        });

        it("should parse YAML arrays", || {
            let yaml = "- 1\n- hello\n- true";
            let parsed = parse_yaml(yaml);
            assert::eq(parsed, [1, "hello", true]);
        });

        it("should convert objects to YAML", || {
            let obj = #{"a": 1, "b": "hello", "c": true};
            let yaml = to_yaml(obj);
            let expected = "a: 1\nb: hello\nc: true\n";
            assert::eq(yaml, expected);
        });
    });

    describe("TOML", || {
        it("should parse basic TOML objects", || {
            let toml = "a = 1\nb = \"hello\"\nc = true";
            let parsed = parse_toml(toml);
            assert::eq(parsed, #{"a": 1, "b": "hello", "c": true});
        });

        it("should parse TOML arrays", || {
            let toml = "array = [1, \"hello\", true]";
            let parsed = parse_toml(toml);
            assert::eq(parsed, #{"array": [1, "hello", true]});
        });

        it("should convert objects to TOML", || {
            let obj = #{"a": 1, "b": "hello", "c": true};
            let toml = to_toml(obj);
            let expected = "a = 1\nb = \"hello\"\nc = true\n";
            assert::eq(toml, expected);
        });
    });

    describe("Error handling", || {
        it("should handle invalid JSON", || {
            try {
                parse_json("{invalid json}");
                assert(false, "Should have thrown error");
            } catch(err) {
                assert(true, "Error caught successfully");
            }
        });

        it("should handle invalid YAML", || {
            try {
                parse_yaml("invalid: yaml: : :");
                assert(false, "Should have thrown error");
            } catch(err) {
                assert(true, "Error caught successfully");
            }
        });

        it("should handle invalid TOML", || {
            try {
                parse_toml("invalid toml = = =");
                assert(false, "Should have thrown error");
            } catch(err) {
                assert(true, "Error caught successfully");
            }
        });
    });
});
