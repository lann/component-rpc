```console
$ cargo install --git https://github.com/bytecodealliance/cargo-component
...
$ ./build-example-and-serve.sh
Parsed world:
interface interface0 {
  add: func(left: u32, right: u32) -> u32
}

world hello-rpc {
  default export interface0
}

Serving on 0.0.0.0:3456
```

```console
$ curl localhost:3456/call --json '{"name": "add", "args": [1, 2]}'
{"result":3}
```
