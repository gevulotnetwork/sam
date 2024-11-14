import "math" as math;
import "mod2" as mod2;

describe("Modules", || {
    it("should be possible to use imported constants", || {
        require(math::PI == 3.141592653589793, "PI should be 3.141592653589793");
    });

    it("should be possible to use imported functions", || {
        require(math::add(1, 2) == 3, "1 + 2 should be 3");
    });

    it("should be possible to use imported constants from other module directories", || {
        require(mod2::CONSTANT_FROM_OTHER_MODULE_DIR == 42, "CONSTANT_FROM_OTHER_MODULE_DIR should be 42");
    });

    it("should be possible to use functions from one module, that import stuff from other modules", || {
        require(math::fourty_two() == 42, "fourty_two() should be 42");
    });
});
