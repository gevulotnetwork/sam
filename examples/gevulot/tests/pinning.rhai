import "gevulot" as gvlt;
import "constants" as constants;

describe("Pinning", || {

    it("should be possible to start the worker components", || {
        start_component("worker-1");
        start_component("worker-2");
    });

    it("should be possible to pin a file", || {
        let cid = "QmZTLASRx5ynWHEmN2BYVhg9tioJw1UjpWnHgdjGmdnyoR";
        let fallback_url = "https://i0.wp.com/picjumbo.com/wp-content/uploads/beautiful-autumn-nature-with-a-river-in-the-middle-of-the-forest-free-image.jpeg?w=600&quality=80";
        
        gvlt::set_identity(constants::ALICE);
        
        let resp = gvlt::ctl(`pin create <<EOF
kind: Pin
version: v0
metadata:
    name: 'Pin 1'
    description: 'A pin with a fallback URL'
    tags: []
    labels: []
spec:
    cid: ${cid}
    fallback_urls: 
    - ${fallback_url}
    bytes: 48264
    time: 3600
    redundancy: 1
EOF
        `);
    });
});
