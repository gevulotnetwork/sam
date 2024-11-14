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

Example Script
-------------

```js
import "module" as mod;

describe("Example Webserver", || {
    it("should be possible to start a component on demand", || {
        start_component("caddy");
        require(true, "Component did not start");
    });

    it("can serve 100 files", || {
        for i in 0..100 {
            let response = exec("curl -s http://localhost:8080/index.html");
            require(response == "Hello, World!", "Caddy did not serve the expected file");
        }
    });
});

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




