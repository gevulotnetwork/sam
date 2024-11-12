import "constants" as constants;

fn ctl(command) {
    return exec("gvltctl " + command);
}

fn set_identity(identity) {
    if identity == constants::ALICE {
        set_env("GEVULOT_MNEMONIC", constants::ALICE_MNEMONIC)
    } else if identity == constants::BOB {
        set_env("GEVULOT_MNEMONIC", constants::BOB_MNEMONIC)
    } else if identity == constants::EVE {
        set_env("GEVULOT_MNEMONIC", constants::EVE_MNEMONIC)
    } else {
        set_env("GEVULOT_MNEMONIC", identity)
    }
}
