import "bignum" as bignum;

describe("BigNum", || {

    it("should be able to parse string numbers", || {
        // Basic parsing
        let a = bignum::parse("100");
        require(bignum::to_string(a) == "100", "Expected 100, got " + bignum::to_string(a));

        // Parse large number
        let b = bignum::parse("999999999999999999999999");
        require(bignum::to_string(b) == "999999999999999999999999", "Expected 999999999999999999999999, got " + bignum::to_string(b));

        // Parse negative number
        let c = bignum::parse("-123456789");
        require(bignum::to_string(c) == "-123456789", "Expected -123456789, got " + bignum::to_string(c));

        // Parse empty string
        let d = bignum::parse("");
        require(bignum::to_string(d) == "0", "Expected 0, got " + bignum::to_string(d));

        // Parse string with whitespace
        let e = bignum::parse("  42  ");
        require(bignum::to_string(e) == "42", "Expected 42, got " + bignum::to_string(e));

        // Parse zero
        let f = bignum::parse("0");
        require(bignum::to_string(f) == "0", "Expected 0, got " + bignum::to_string(f));

        // Parse negative large number
        let g = bignum::parse("-999999999999999999999999");
        require(bignum::to_string(g) == "-999999999999999999999999", "Expected -999999999999999999999999, got " + bignum::to_string(g));

        // Parse number with leading zeros
        let h = bignum::parse("00001234");
        require(bignum::to_string(h) == "1234", "Expected 1234, got " + bignum::to_string(h));

        // Parse number with trailing zeros
        let i = bignum::parse("12340000");
        require(bignum::to_string(i) == "12340000", "Expected 12340000, got " + bignum::to_string(i));

        // Parse number with mixed whitespace
        let j = bignum::parse("  00001234  ");
        require(bignum::to_string(j) == "1234", "Expected 1234, got " + bignum::to_string(j));
    });

    it("should be able to add two big numbers", || {
        // Basic addition
        let a = bignum::parse("100000000000000000000000");
        let b = bignum::parse("200000000000000000000000");
        let c = bignum::add(a, b);
        require(bignum::to_string(c) == "300000000000000000000000", "Expected 300000000000000000000000, got " + bignum::to_string(c));

        // Addition with overflow from low to high
        let a = bignum::parse("999999999999999999999999");
        let b = bignum::parse("1");
        let c = bignum::add(a, b);
        require(bignum::to_string(c) == "1000000000000000000000000", "Expected 1000000000000000000000000, got " + bignum::to_string(c));

        // Addition with negative numbers
        let a = bignum::parse("-100000000000000000000000");
        let b = bignum::parse("300000000000000000000000");
        let c = bignum::add(a, b);
        require(bignum::to_string(c) == "200000000000000000000000", "Expected 200000000000000000000000, got " + bignum::to_string(c));

        // Addition resulting in zero
        let a = bignum::parse("500000000000000000000000");
        let b = bignum::parse("-500000000000000000000000");
        let c = bignum::add(a, b);
        require(bignum::to_string(c) == "0", "Expected 0, got " + bignum::to_string(c));

        // Addition with both numbers negative
        let a = bignum::parse("-100000000000000000000000");
        let b = bignum::parse("-200000000000000000000000");
        let c = bignum::add(a, b);
        require(bignum::to_string(c) == "-300000000000000000000000", "Expected -300000000000000000000000, got " + bignum::to_string(c));

        // Addition with zero
        let a = bignum::parse("0");
        let b = bignum::parse("123456789012345678901234");
        let c = bignum::add(a, b);
        require(bignum::to_string(c) == "123456789012345678901234", "Expected 123456789012345678901234, got " + bignum::to_string(c));

        // Addition with large numbers
        let a = bignum::parse("999999999999999999999999999999");
        let b = bignum::parse("999999999999999999999999999999");
        let c = bignum::add(a, b);
        require(bignum::to_string(c) == "1999999999999999999999999999998", "Expected 1999999999999999999999999999998, got " + bignum::to_string(c));
    });

    it("should be able to subtract two big numbers", || {
        // Basic subtraction
        let a = bignum::parse("200000000000000000000000");
        let b = bignum::parse("100000000000000000000000");
        let c = bignum::sub(a, b);
        require(bignum::to_string(c) == "100000000000000000000000", "Expected 100000000000000000000000, got " + bignum::to_string(c));

        // Subtraction with underflow from high to low
        let a = bignum::parse("1000000000000000000000000");
        let b = bignum::parse("1");
        let c = bignum::sub(a, b);
        require(bignum::to_string(c) == "999999999999999999999999", "Expected 999999999999999999999999, got " + bignum::to_string(c));

        // Subtraction resulting in negative
        let a = bignum::parse("100000000000000000000000");
        let b = bignum::parse("200000000000000000000000");
        let c = bignum::sub(a, b);
        require(bignum::to_string(c) == "-100000000000000000000000", "Expected -100000000000000000000000, got " + bignum::to_string(c));

        // Subtraction with zero
        let a = bignum::parse("123456789012345678901234");
        let b = bignum::parse("0");
        let c = bignum::sub(a, b);
        require(bignum::to_string(c) == "123456789012345678901234", "Expected 123456789012345678901234, got " + bignum::to_string(c));

        // Subtraction resulting in zero
        let a = bignum::parse("500000000000000000000000");
        let b = bignum::parse("500000000000000000000000");
        let c = bignum::sub(a, b);
        require(bignum::to_string(c) == "0", "Expected 0, got " + bignum::to_string(c));

        // Subtraction with both numbers negative
        let a = bignum::parse("-300000000000000000000000");
        let b = bignum::parse("-100000000000000000000000");
        let c = bignum::sub(a, b);
        require(bignum::to_string(c) == "-200000000000000000000000", "Expected -200000000000000000000000, got " + bignum::to_string(c));

        // Subtraction with large numbers
        let a = bignum::parse("999999999999999999999999999999");
        let b = bignum::parse("888888888888888888888888888888");
        let c = bignum::sub(a, b);
        require(bignum::to_string(c) == "111111111111111111111111111111", "Expected 111111111111111111111111111111, got " + bignum::to_string(c));
    });

    it("should be able to compare two big numbers", || {
        // Basic comparisons
        let a = bignum::parse("100000000000000000000000");
        let b = bignum::parse("200000000000000000000000");
        require(bignum::cmp(a, b) == -1, "Expected -1, got " + bignum::cmp(a, b).to_string());
        
        let a = bignum::parse("100000000000000000000000");
        let b = bignum::parse("100000000000000000000000");
        require(bignum::cmp(a, b) == 0, "Expected 0, got " + bignum::cmp(a, b).to_string());
        
        let a = bignum::parse("200000000000000000000000");
        let b = bignum::parse("100000000000000000000000");
        require(bignum::cmp(a, b) == 1, "Expected 1, got " + bignum::cmp(a, b).to_string());

        // Compare with negative numbers
        let a = bignum::parse("-100000000000000000000000");
        let b = bignum::parse("100000000000000000000000");
        require(bignum::cmp(a, b) == -1, "Expected -1, got " + bignum::cmp(a, b).to_string());

        // Compare numbers with same high but different low
        let a = bignum::parse("1000000000000000000001");
        let b = bignum::parse("1000000000000000000000");
        require(bignum::cmp(a, b) == 1, "Expected 1, got " + bignum::cmp(a, b).to_string());

        // Additional test cases
        // Compare two negative numbers
        let a = bignum::parse("-200000000000000000000000");
        let b = bignum::parse("-100000000000000000000000");
        require(bignum::cmp(a, b) == -1, "Expected -1, got " + bignum::cmp(a, b).to_string());

        // Compare negative and zero
        let a = bignum::parse("-100000000000000000000000");
        let b = bignum::parse("0");
        require(bignum::cmp(a, b) == -1, "Expected -1, got " + bignum::cmp(a, b).to_string());

        // Compare zero and positive
        let a = bignum::parse("0");
        let b = bignum::parse("100000000000000000000000");
        require(bignum::cmp(a, b) == -1, "Expected -1, got " + bignum::cmp(a, b).to_string());

        // Compare zero and zero
        let a = bignum::parse("0");
        let b = bignum::parse("0");
        require(bignum::cmp(a, b) == 0, "Expected 0, got " + bignum::cmp(a, b).to_string());

        // Compare large numbers with different high parts
        let a = bignum::parse("999999999999999999999999999999");
        let b = bignum::parse("888888888888888888888888888888");
        require(bignum::cmp(a, b) == 1, "Expected 1, got " + bignum::cmp(a, b).to_string());
    });

});
