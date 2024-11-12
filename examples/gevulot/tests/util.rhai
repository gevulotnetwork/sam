import "bignum" as bignum;
import "gevulot" as gvlt;

fn get_balance(account) {
    let str = gvlt::ctl(`account-info ${account} --format json`);
    let obj = parse_json(str);
    let balance = obj.balance;
    return bignum::parse(balance);
};