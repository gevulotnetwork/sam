describe("Example Webserver", || {
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

    it("should be possible to sleep", || {
        sleep("100ms");
        require(true, "Sleep did not work");
    });

    it("should be possible to wait until a condition is met", || {
        let now = timestamp();
        wait_until(|| { 
            now.elapsed() > 0.5
        }, "1s");
        require(true, "Wait until did not work");
    });
});
