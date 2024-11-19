import "assert" as assert;
import "require" as require;

describe("stdlib", || {
    describe("assert", || {
        it("should be able to assert", || {
            assert::eq(1, 1);
            assert::ne(1, 2);
            assert::gt(2, 1);
            assert::ge(2, 2);
            assert::lt(1, 2);
            assert::le(1, 1);
        });
        it("should be possible to assert with custom message", || {
            assert::eq(1, 1, "This is a custom message");
        });
        it("should be possible to assert any type", || {
            assert::eq("Hello", "Hello");
            assert::ne("Hello", "World");
        });
    });


    describe("require", || {
        it("should be able to require", || {
            require::eq(1, 1);
            require::ne(1, 2);
            require::gt(2, 1);
            require::ge(2, 2);
            require::lt(1, 2);
            require::le(1, 1);
        });
        it("should be possible to require with custom message", || {
            require::eq(1, 1, "This is a custom message");
        });
        it("should be possible to require any type", || {
            require::eq("Hello", "Hello");
            require::ne("Hello", "World");
        });
    });

    describe("assert_eq", || {
        it("should be able to assert_eq", || {
            assert::eq(#{"a": 1, "b": 2}, #{"a": 1, "b": 2});
        });
    });

});
