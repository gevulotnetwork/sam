const TEST_DIR = ".smoke-test";
const CONFIG_FILE = TEST_DIR + "/config.yaml";
const EXAMPLE_FILE = TEST_DIR + "/tests/example.rhai";

const EXAMPLE_CONFIG = `
name: smoke-test
global:
  scripts:
    - .smoke-test/tests/example.rhai`;

const EXAMPLE_TEST = `
describe("Example Test", || {
    it("should be possible to print hello world", || {
        log("Hello, World!");
    });
});`;

describe("Smoke Test", || {
    it("should be possible to create a minimal test environment", || {
        exec(`mkdir -p ${TEST_DIR}/tests`);
        exec(`cat > ${CONFIG_FILE} <<EOF
${EXAMPLE_CONFIG}
EOF`);
        exec(`cat > ${EXAMPLE_FILE} <<EOF
${EXAMPLE_TEST}
EOF`);

        let output = exec(`./target/release/sam -c ${CONFIG_FILE}`);
        require(output.contains("succeeded"), "Expected output to contain 'succeeded'");
    });
});
