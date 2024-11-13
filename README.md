SAM - Simple Automation Manager
===============================

SAM is a simple yet powerful integration test framework designed for testing distributed systems and components. It provides a flexible way to define test environments, manage components, and write expressive tests using the Rhai scripting language.

Key Features
-----------
- Define test environments in YAML with support for pods, containers and raw processes
- Write tests in Rhai with familiar describe/it syntax and assertions
- Control components with start/stop commands and dependency management
- Built-in test utilities
- Filter and skip tests using expressions
- Repeat test runs
- Configurable delays
- Keep environment running after tests complete
- Module system for organizing test code
- Detailed test reporting with timing and status information

Quick Start
----------
0. Have some assets to play with:
```sh
mkdir -p assets/www
echo "Hello, World\!" > assets/www/index.html
```

1. Create a config.yaml file to define your test environment and save it as `config.yaml`:
```yaml
name: "basic"

components:
  - name: "caddy"
    type: container
    start_by_default: true
    image: "caddy:latest"
    ports:
      - host: 8080
        container: 80
    command: ["caddy", "file-server", "--root", "/srv/www"]
    volumes:
      - host: "./assets/www"
        container: "/srv/www"

global:
  scripts:
    - "./tests.rhai"
```

2. Write your tests in Rhai using the describe/it syntax and save it as `tests.rhai`:
```js
describe("Example Webserver", || {
    it("serves files", || {
        let response = exec("curl -s http://localhost:8080/index.html");
        require(response == "Hello, World!\n", "Caddy did not serve the file");
    });
});
```

3. Run the tests:
```sh
sam run
```

Available Functions
-----------------

SAM provides several global functions that can be used in your test scripts:

### Test Organization

- `describe(name: string, callback: function)` - Groups related tests together under a descriptive name. The callback contains the test cases.

- `it(name: string, callback: function)` - Defines an individual test case with a descriptive name. The callback contains the test logic.

- `require(condition: bool, message: string)` - Asserts that a condition is true. If false, fails the test with the provided error message.

### Command Execution

- `exec(command: string) -> string` - Executes a shell command and returns its stdout output as a string. Throws an error if the command fails or returns a non-zero exit code.

### Environment Control

- `start_component(name: string)` - Starts a component defined in the config file by name.

- `stop_component(name: string)` - Stops a running component by name.

- `set_env(key: string, value: string)` - Sets an environment variable that will be available to subsequent commands.

### Flow Control

- `sleep(duration: string)` - Pauses execution for the specified duration. Duration should be in human-readable format (e.g. "1s", "500ms", "2m").

- `wait_until(condition: function, timeout: string|int)` - Waits until the condition function returns true or the timeout is reached. Timeout can be specified as:
  - A human-readable duration string (e.g. "30s")
  - Number of milliseconds as integer

### Logging

- `log(message: string)` - Logs a message to the console with the current file and line number.




