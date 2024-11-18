// SAM STDLIB: assert

fn eq(a, b) {
    let diff = diff(a.to_string(), b.to_string());
    eq(a, b, `Expected ${a} to be equal to ${b}` + "\n" + diff);
}

fn eq(a, b, msg) {
    assert(a == b, msg);
}

fn ne(a, b) {
    ne(a, b, `Expected ${a} to not be equal to ${b}`);
}

fn ne(a, b, msg) {
    assert(a != b, msg);
}

fn gt(a, b) {
    gt(a, b, `Expected ${a} to be greater than ${b}`);
}

fn gt(a, b, msg) {
    assert(a > b, msg);
}

fn ge(a, b) {
    ge(a, b, `Expected ${a} to be greater than or equal to ${b}`);
}

fn ge(a, b, msg) {
    assert(a >= b, msg);
}

fn lt(a, b) {
    lt(a, b, `Expected ${a} to be less than ${b}`);
}

fn lt(a, b, msg) {
    assert(a < b, msg);
}

fn le(a, b) {
    le(a, b, `Expected ${a} to be less than or equal to ${b}`);
}

fn le(a, b, msg) {
    assert(a <= b, msg);
}
