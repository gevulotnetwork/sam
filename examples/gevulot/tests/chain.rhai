import "util" as util;
import "bignum" as bignum;
import "gevulot" as gvlt;
import "constants" as constants;

const ALICE = constants::ALICE;
const BOB = constants::BOB;
const EVE = constants::EVE;

describe("Default Chain Setup", || {

    describe("Default Containers are running", || {

        it("should have ember running", || {
            let resp = exec("podman container ps -a -f name=e2e-ember --format json| jq -r '.[0].State'");
            require(resp == "running\n", "Expected ember to be running, got " + resp);
        });

    });


    describe("Worker Component Startup", || {
        it("should be possible to start the worker components", || {
            start_component("worker-1");
            start_component("worker-2");
        });


        it("should have postgres running", || {
            let resp = exec("podman container ps -a -f name=e2e-postgres --format json| jq -r '.[0].State'");
            require(resp == "running\n", "Expected postgres to be running, got " + resp);
        });

        it("should have ipfs running", || {
            let resp = exec("podman container ps -a -f name=e2e-ipfs --format json| jq -r '.[0].State'");
            require(resp == "running\n", "Expected ipfs to be running, got " + resp);
        });
        
        it("should have sara-1 running", || {
            let resp = exec("podman container ps -a -f name=e2e-sara-1 --format json| jq -r '.[0].State'");
            require(resp == "running\n", "Expected sara-1 to be running, got " + resp);
        });

        it("should have eve-1 running", || {
            let resp = exec("podman container ps -a -f name=e2e-eve-1 --format json| jq -r '.[0].State'");
            require(resp == "running\n", "Expected eve-1 to be running, got " + resp);
        });

        it("should have sara-2 running", || {
            let resp = exec("podman container ps -a -f name=e2e-sara-2 --format json| jq -r '.[0].State'");
            require(resp == "running\n", "Expected sara-2 to be running, got " + resp);
        });

        it("should have eve-2 running", || {
            let resp = exec("podman container ps -a -f name=e2e-eve-2 --format json| jq -r '.[0].State'");
            require(resp == "running\n", "Expected eve-2 to be running, got " + resp);
        });

        it("should have nomad running", || {
            let resp = exec("ps -ef|grep nomad | wc -l");
            require(resp == "3\n", "Expected nomad to be running, got " + resp);
        });
    });
    
    describe("Initial Chain State", || {
        it("should have 2 workers", || {
            let resp = gvlt::ctl("worker list --format json | jq -r '.[].metadata.name' | wc -l");
            require(resp == "2\n", "Expected 2 worker, got " + resp);
        });
    
        it("should have the alice, bob and eve accounts", || {
            let alice_info = gvlt::ctl(`account-info ${ALICE}`);
            let bob_info = gvlt::ctl(`account-info ${BOB}`);
            let eve_info = gvlt::ctl(`account-info ${EVE}`);
            require(alice_info != "", "Expected alice account, got " + alice_info);
            require(bob_info != "", "Expected bob account, got " + bob_info);
            require(eve_info != "", "Expected eve account, got " + eve_info);
        });
    });

    describe("Sending Money", || {
        it("should be possible to send money from alice to bob", || {
            let alice_balance_before = util::get_balance(ALICE);
            let bob_balance_before = util::get_balance(BOB);

            gvlt::set_identity(ALICE);
            gvlt::ctl(`send 100 ${BOB}`);

            let alice_balance_after = util::get_balance(ALICE);
            let bob_balance_after = util::get_balance(BOB);

            let hundred = bignum::parse("100");
            let alice_expected = bignum::sub(alice_balance_before, hundred);

            // alice should have less than 100 less than before because of the gas fees
            require(bignum::cmp(alice_balance_after, alice_expected) < 0, 
                   "Expected alice to have less balance, got " + bignum::to_string(alice_balance_after));

            // bob should have exactly 100 more than before
            let bob_diff = bignum::sub(bob_balance_after, bob_balance_before);
            require(bignum::cmp(bob_diff, hundred) == 0,
                   "Expected bob to have more balance, got " + bignum::to_string(bob_balance_after));
        });
    });
    
});

