🚀 SAM - Simple Automation Manager! 
===============================

Ever wished testing distributed systems could be as fun as playing with LEGO? 🎮 Meet SAM - your new best friend in the testing world! 🤖

Like a skilled conductor orchestrating a symphony 🎵, SAM brings harmony to your integration tests by magically managing your distributed components. With its friendly Rhai scripting language and powerful features, SAM transforms the complex chaos of system testing into a delightful journey. Whether you're a testing novice or a seasoned pro, SAM makes testing feel less like a chore and more like an adventure! ✨

SAM takes care of all the heavy lifting - spinning up containers, managing processes, handling dependencies, and coordinating test flows - so you can focus on what matters: writing awesome tests! Think of it as your personal testing assistant, ready to handle the complex orchestration while you craft beautiful test scenarios. With an intuitive YAML configuration and expressive Rhai scripting, SAM makes distributed system testing accessible, maintainable, and dare we say... fun! 🎯 

No more wrestling with brittle test setups or pulling your hair out over flaky integration tests. SAM's got your back with powerful features like test filtering, configurable delays, dependency management, and detailed reporting. Ready to revolutionize your testing workflow? Let's dive in! 🚀

✨ Magical Powers ✨
-----------------
- 🎮 Build your test playground in YAML - bring pods, containers, and processes to life!
- 🎯 Write super-friendly tests in Rhai that read like a story
- 🎭 Play puppet master with your components - start, stop, and manage them like a pro
- 🧰 Packed with handy testing tools right out of the box
- 🔍 Cherry-pick your tests with smart filtering - run exactly what you want
- 🔄 Need another go? Repeat tests as many times as you like
- ⏰ Add dramatic pauses with configurable delays
- 🏃‍♂️ Keep your test environment alive and kicking after the show
- 📦 Organize your test code into neat little modules
- 📊 Get beautiful test reports with all the juicy details

🎮 Let's Play!
-------------
0. First, let's set up your magical testing playground and run your first adventure! 🚀
```sh
sam init # This will create a basic sam.yaml and a directory structure for your tests
sam run # This will run the tests
```

1. Now look at your `sam.yaml` file to get a feeling for how it works:
```yaml
name: example-test
global:
  scripts: # scripts to run
    - tests/cases/example.rhai
  module_dirs: # directories to load modules from
    - tests/modules
  delay: "1s" # delay between test runs
  repeat: 2 # repeat the tests this many times
  filter: "" # filter tests using a regular expression
  skip: "" # skip tests using a regular expression
  reset_once: false # reset the environment once before running tests
  force: false # force the environment to be reset before running tests
  keep_running: false # keep the environment running after tests complete

# Components are processes, containers or pods that are started and stopped by SAM
components:
  - name: caddy
    type: container # type of component
    start_by_default: true # start the component by default
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

# Reset is a list of commands to run when resetting the environment to restore it to a known state
reset:
  - echo 'Reverts assets...'
  - echo 'hello world' > tests/assets/hello.txt

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

3. Its using a module called `example` that is defined in the `tests/modules/example.rhai` file. It's a wrapper around the global `http_get` function.
```js
export const URL = "http://127.0.0.1:8080/hello.txt";

// functions are always exported
fn fetch(url) {
    http_get(#{url: url});
}
```

🎯 Example Output 🚀
------------

```text
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

🛠️ Available Functions & Utilities 🧰
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

The HTTP functions accept an options object with the following properties:

- `url: string` - Required. The URL to make the request to
- `params: object` - Optional. Query parameters to append to URL (e.g. `{"key": "value"}` becomes `?key=value`) 
- `headers: object` - Optional. Headers to include in request (e.g. `{"Content-Type": "application/json"}`)
- `body: string` - Optional. Request body (only for POST requests)

Available functions:

- `http_get(options: Dynamic) -> string` - Makes HTTP GET request and returns response body
- `http_post(options: Dynamic) -> string` - Makes HTTP POST request and returns response body
- `http_head(options: Dynamic)` - Makes HTTP HEAD request to check if resource exists

Example:
```js
http_get(#{
  url: "http://127.0.0.1:8080/hello.txt", 
  headers: {"Content-Type": "application/json"}
});
```

### Math/Random

- `random_string(length: int) -> string` - Generates random string
- `random_int(min: int, max: int) -> int` - Generates random integer

### Concurrency

- `spawn_task(callback: function) -> int` - Spawns async task, returns task ID
- `wait_for_tasks(ids: Array) -> Array` - Waits for multiple tasks to complete
- `wait_for_task(id: int) -> Dynamic` - Waits for single task to complete

