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
0. Init your test environment and run the first test!
```sh
sam init # This will create a basic sam.yaml and a directory structure for your tests
sam run # This will run the tests
```

1. Now look at your `sam.yaml` file to get a feeling for how it works:
```yaml
name: example-test
global:
  scripts:
    - tests/cases/example.rhai
  module_dirs:
    - tests/modules
  delay: null
  repeat: null
  filter: null
  skip: null
  reset_once: false
  force: false
  keep_running: false

components:
  - name: caddy
    type: container
    start_by_default: true
    image: docker.io/library/caddy:latest
    command: ["caddy", "file-server", "--root", "/srv"]
    ports:
      - host: 8080
        container: 80
    volumes:
      - host: ./tests/assets
        container: /srv
    environment:
      - CADDY_ADMIN_PORT=2019

reset:
  - echo 'Reverts assets...'
  - echo 'hello world' > tests/assets/hello.txt%    

```

2. See that it references the `example.rhai` script:
```js
import "example" as example;

describe("Example Test Suite", || {
    it("should respond to HTTP requests", || {
        // Make request to Caddy server
        let response = example::fetch(example::URL);     
        // Validate response
        assert(response == "hello world", "Expected valid response from server");
    });
});
```

Example Output
------------

```bash
 INFO  sam >                    _____ _____ _____
 INFO  sam >                   |   __|  _  |     |
 INFO  sam >                   |__   |     | | | |
 INFO  sam >                   |_____|__|__|_|_|_|
 INFO  sam >                 Simple Automation Manager
 INFO  sam > Welcome to SAM, your simple and friendly test automation manager!
 INFO  sam >
 INFO  sam > Resetting environment
 INFO  sam::environment > Starting environment...
 INFO  sam::environment > Environment started successfully in 1s 268ms 365us 387ns
 INFO  sam::rhai        > Running script file examples/gevulot/tests/chain.rhai
 TEST  Testing Default Chain Setup ...
 TEST    Testing Default Containers are running ...
 TEST      It should have ember running... ✅ (49ms 870us 188ns)
 TEST    Testing Default Containers are running succeeded! ✅ (1 tests passed) (49ms 942us 502ns)
 TEST    Testing Worker Component Startup ...
 TEST      It should be possible to start the worker components... ✅ (2s 583ms 857us 119ns)
 TEST      It should have postgres running... ✅ (50ms 614us 948ns)
 TEST      It should have ipfs running... ✅ (54ms 132us 322ns)
 TEST      It should have sara-1 running... ✅ (51ms 133us 641ns)
 TEST      It should have eve-1 running... ✅ (50ms 371us 338ns)
 TEST      It should have sara-2 running... ✅ (49ms 633us 975ns)
 TEST      It should have eve-2 running... ✅ (50ms 207us 382ns)
 TEST      It should have nomad running... ✅ (14ms 839us 839ns)
 TEST    Testing Worker Component Startup succeeded! ✅ (8 tests passed) (2s 905ms 317us 485ns)
 TEST    Testing Initial Chain State ...
 TEST      It should have 2 workers... ✅ (8ms 469us 407ns)
 TEST      It should have the alice, bob and eve accounts... ✅ (27ms 331us 726ns)
 TEST    Testing Initial Chain State succeeded! ✅ (2 tests passed) (35ms 887us 414ns)
 TEST    Testing Sending Money ...
 TEST      It should be possible to send money from alice to bob... ✅ (1s 52ms 517us 877ns)
 TEST    Testing Sending Money succeeded! ✅ (1 tests passed) (1s 52ms 569us 268ns)
 TEST  Testing Default Chain Setup succeeded! ✅ (12 tests passed) (4s 43ms 764us 73ns)
 INFO  sam::environment > Stopping environment...
 INFO  sam::environment > Environment stopped successfully in 3s 52ms 172us 584ns
 INFO  sam              > Run completed in 9s 489ms 729us 722ns
```

Available Functions
-----------------

SAM provides several global functions that can be used in your test scripts:

### Test Organization and Utilities

- `describe(name: string, callback: function)` - Groups related tests together under a descriptive name. The callback contains the test cases. Alias: `task`
- `it(name: string, callback: function)` - Defines an individual test case with a descriptive name. The callback contains the test logic. Alias: `step`
- `require(condition: bool, message: string)` - Asserts that a condition is true. If false, fails the test with the provided error message
- `assert(condition: bool, message: string)` - Similar to require but continues test execution on failure
- `diff(expected: string, actual: string) -> string` - Returns a diff between two strings

### System Commands

- `exec(command: string) -> string` - Executes a shell command and returns its stdout output
- `start_component(name: string)` - Starts a component defined in the config file
- `stop_component(name: string)` - Stops a running component
- `set_env(key: string, value: string)` - Sets an environment variable
- `get_env(key: string) -> string` - Gets value of environment variable
- `sleep(duration: string)` - Pauses execution for specified duration (e.g. "1s", "500ms")
- `wait_until(condition: function, timeout: string|int)` - Waits for condition to return true
- `log(message: string)` - Logs a message to console

### Key-Value Store

- `get(key: string) -> Dynamic` - Gets a value from the shared key-value store
- `set(key: string, value: Dynamic)` - Sets a value in the shared key-value store

### Encoding

- `parse_json(json: string) -> Dynamic` - Parses JSON string into object
- `parse_yaml(yaml: string) -> Dynamic` - Parses YAML string into object  
- `parse_toml(toml: string) -> Dynamic` - Parses TOML string into object
- `to_json(value: Dynamic) -> string` - Converts value to JSON string
- `to_json_pretty(value: Dynamic) -> string` - Converts value to pretty-printed JSON
- `to_yaml(value: Dynamic) -> string` - Converts value to YAML string
- `to_toml(value: Dynamic) -> string` - Converts value to TOML string

### File System

- `temp_dir(prefix: string) -> string` - Creates temporary directory with prefix
- `write_file(path: string, content: string)` - Writes content to file
- `read_file(path: string) -> string` - Reads content from file
- `mkdir(path: string)` - Creates directory
- `remove(path: string)` - Removes file or directory
- `ls(path: string) -> Array` - Lists directory contents
- `file_exists(path: string) -> bool` - Checks if file exists
- `stat(path: string) -> Dynamic` - Gets file metadata
- `copy(src: string, dst: string)` - Copies file
- `rename(src: string, dst: string)` - Renames/moves file
- `is_dir(path: string) -> bool` - Checks if path is directory
- `is_file(path: string) -> bool` - Checks if path is file
- `absolute_path(path: string) -> string` - Gets absolute path

### HTTP

- `http_get(options: Dynamic) -> string` - Makes HTTP GET request
- `http_post(options: Dynamic) -> string` - Makes HTTP POST request  
- `http_head(options: Dynamic)` - Makes HTTP HEAD request

### Math/Random

- `random_string(length: int) -> string` - Generates random string
- `random_int(min: int, max: int) -> int` - Generates random integer

### Concurrency

- `spawn_task(callback: function) -> int` - Spawns async task, returns task ID
- `wait_for_tasks(ids: Array) -> Array` - Waits for multiple tasks to complete
- `wait_for_task(id: int) -> Dynamic` - Waits for single task to complete

