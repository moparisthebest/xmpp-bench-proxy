xmpp-bench-proxy
----------------

[![Build Status](https://ci.moparisthe.best/job/moparisthebest/job/xmpp-bench-proxy/job/master/badge/icon%3Fstyle=plastic)](https://ci.moparisthe.best/job/moparisthebest/job/xmpp-bench-proxy/job/master/)

[rtb](https://github.com/processone/rtb) and [tsung](https://github.com/processone/tsung) are nice projects to benchmark XMPP servers, but they don't support connecting via WebSocket.  This simply translates plaintext C2S XMPP to TLS WebSocket, quickly, using [xmpp-proxy](https://github.com/moparisthebest/xmpp-proxy).  It explicitly doesn't do any DNS or certificate validation, so only use it for benchmarking.  If you need that, use xmpp-proxy directly.

####  License
GNU/AGPLv3 - Check LICENSE.md for details
