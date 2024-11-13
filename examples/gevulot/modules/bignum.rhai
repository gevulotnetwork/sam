fn BigNum(low, high) {
    #{
        low: low,
        high: high
    }
}

fn parse(s) {
    s.trim();
    if s.len == 0 {
        return BigNum(0, 0);
    }

    let negative = s.starts_with("-");
    if negative {
        s = s.sub_string(1);
    }

    // Split into chunks of 18 digits (max safe for i64)
    let low = if s.len <= 18 {
        s.parse_int()
    } else {
        s.sub_string(s.len - 18).parse_int()
    };

    let high = if s.len <= 18 {
        0
    } else {
        s.sub_string(0, s.len - 18).parse_int()
    };

    if negative {
        BigNum(-low, -high)
    } else {
        BigNum(low, high)
    }
}

fn add(a, b) {
    let low = a.low + b.low;
    let high = a.high + b.high;
    
    // Check if we need to carry over to high part
    if low >= 1_000_000_000_000_000_000 {
        low -= 1_000_000_000_000_000_000;
        high += 1;
    } else if low <= -1_000_000_000_000_000_000 {
        low += 1_000_000_000_000_000_000;
        high -= 1;
    }
    
    BigNum(low, high)
}

fn sub(a, b) {
    let low = a.low - b.low;
    let high = a.high - b.high;
    
    // Handle underflow from low to high
    if low < 0 {
        low += 1_000_000_000_000_000_000;
        high -= 1;
    } else if low >= 1_000_000_000_000_000_000 {
        low -= 1_000_000_000_000_000_000;
        high += 1;
    }
    
    BigNum(low, high)
}

fn cmp(a, b) {
    if a.high != b.high {
        if a.high < b.high {
            return -1;
        }
        return 1;
    }
    if a.low != b.low {
        if a.low < b.low {
            return -1;
        }
        return 1;
    }
    return 0;
}

fn to_string(num) {
    if num.high == 0 {
        return num.low.to_string();
    }
    
    // Need to handle negative numbers
    if num.high < 0 {
        return "-" + to_string(BigNum(-num.low, -num.high));
    }
    
    // Need proper zero padding for place values
    let low_str = num.low.abs().to_string();
    let low_padded = "";
    for i in 0..(18 - low_str.len) {
        low_padded += "0";
    }
    low_padded += low_str;
    return num.high.to_string() + low_padded;
}


