describe("loading config with base", || {
    it("should be possible to start a component defined in the base", ||{
        start_component("test-container");
    });
    it("should be possible to start a component defined in the extension", ||{
        start_component("test-container2");
    });
});
