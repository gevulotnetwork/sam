import "mod2" as mod2;

export const PI = 3.141592653589793;

fn add(a, b) {
    return a + b;
}

fn fourty_two() {
    return mod2::CONSTANT_FROM_OTHER_MODULE_DIR;
}

