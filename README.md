# `hyper` memory leak

This is an example for a very specific situation in which I got a memory leak using `hyper` as a reverse proxy:

- have a reverse proxy listening via HTTP/2.
- forward requests to an upstream server via HTTP/1.1
- take the response from the upstream and forward it to the original client

Only this combination "leaks" memory from the `HeaderValue`s. It seems that it is not a real leak, but somehow bound to
the underlying HTTP/2 connection, but I am not sure about it yet. It does not make any difference if it's plain HTTP or
TLS-encrypted. The behavior only depends on the proxy connection being HTTP/2 and the upstream HTTP/1.1. Also, the more
concurrent connections you have, the faster memory usage increases.

This example will not work in a normal browser because it only accepts HTTP/2, but I wanted to keep the code as minimal
as possible. All other combinations of proxy / upstream do not leak memory, only this one. The longer the test runs, the
slower the used memory will increase.

To test the leak, use any load-generating tool like [oha](https://github.com/hatoo/oha). To see the results quicker, do
not use it on the debug binary:

```bash
cargo build --release
``` 

Then:

```bash
./target/release/hyper-mem-leak
```

To generate load:

```bash
oha --http2 -n 10000000 -c 100 http://localhost:8000
```

You will see the memory for the process going up steadily. To see the effect of the `HeaderValue` re-allocation, restart
the binary with:

```bash
./target/release/hyper-mem-leak realloc
```

and generate load again. It should be stable once all buffers are fully used.
