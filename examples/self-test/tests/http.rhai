
describe("HTTP", || {
    it("should be able to get a url", || {
        let result = parse_json(http_get(#{"url": "https://httpbin.org/get"}));
        assert(result.get("url") == "https://httpbin.org/get", "HTTP get returned unexpected result");
    });

    it("should be possible to add parameters to get requests", || {
        let result = parse_json(http_get(#{"url": "https://httpbin.org/get", "params": #{"a": "1", "b": "2"}}));
        assert(result.get("args") == #{"a": "1", "b": "2"}, "HTTP get returned unexpected result");
    });

    it("should be possible to add headers to get requests", || {
        let result = parse_json(http_get(#{"url": "https://httpbin.org/get", "headers": #{"X-Test": "123"}}));
        assert(result.get("headers").get("X-Test") == "123", "HTTP get returned unexpected result");
    });

    it("should be possible to add multiple headers to get requests", || {
        let result = parse_json(http_get(#{"url": "https://httpbin.org/get", "headers": #{"X-Test": "123", "X-Test2": "456"}}));
        assert(result.get("headers").get("X-Test") == "123", "HTTP get returned unexpected result");
        assert(result.get("headers").get("X-Test2") == "456", "HTTP get returned unexpected result");
    });

    it("should be able to post to a url", || {
        let result = parse_json(http_post(#{"url": "https://httpbin.org/post", "body": "Hello, World!"}));
        assert(result.get("data") == "Hello, World!", "HTTP post returned unexpected result");
    });

    it("should be possible to add headers to post requests", || {
        let result = parse_json(http_post(#{"url": "https://httpbin.org/post", "headers": #{"X-Test": "123"}}));
        assert(result.get("headers").get("X-Test") == "123", "HTTP post returned unexpected result");
    });

    it("should be possible to add parameters to the url", || {
        let result = parse_json(http_get(#{"url": "https://httpbin.org/get?a=1&b=2"}));
        assert(result.get("args") == #{"a": "1", "b": "2"}, "HTTP get returned unexpected result");
    });

    it("should be able to head a url", || {
        http_head(#{"url": "https://httpbin.org/get"});
        assert(true, "HTTP head returned successfully");
    });

});