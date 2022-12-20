```console
$ cargo install --git https://github.com/bytecodealliance/cargo-component
...
$ ./build-example-and-serve.sh
```

```console
$ curl localhost:3456/call --json '{"name": "decode-hex", "args": ["abcd"]}'
{"result":43981}
```
