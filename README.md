### mqttea
[MQTT5.0](https://docs.oasis-open.org/mqtt/mqtt/v5.0/mqtt-v5.0.html)

## Some Guides (Incomplete)
1. To generate the `wasm` build:
    - Ensure you have [`wasm-pack`](https://github.com/rustwasm/wasm-pack) installed on your local machine
    NB: Because [wasm-pack does not seem to support cargo-workspace](https://github.com/rustwasm/wasm-pack/issues/642)
        - Cd into the core directory: In this case `mqttea-core` first
```
> cd mqttea-core
> wasm-pack build --target web --out-dir ./../pkg
```
    - This should generate the `pkg` folder which you can use for your javascript/typescript projects at the root level of this project

2. To generate `asm` output:
```
> cd mqttea-core
> cargo rustc --lib -- --emit asm
OR 
> cargo rustc --lib --release -- --emit asm #for prod
```
    - you would find the `assembly output`inside `mqttea/target/debug/deps/mqttea_core.s` (if you ran the command above in debug mode, else it would be in the release folder(insteads of deps) if ran with release)


## Future Goals
- [ ] Security First Architecture
    - [ ] Identity & Transport
        - [ ] Mandatory TLS 1.3
        - [ ] Mutual TLS (mTLS) with automatic certificate rotation
        - [ ] Client identity attestation
            - [ ] Cryptographically bound identities
            - [ ] Hardware-backed identity hooks (TPM / HSM ready)
            - Pluggable identity providers (PKI, SPIFEE, OIDC)


### Current Plans
- [ ] Protocol Support
    - [ ] MQTT 3.1.1
    - [ ] MQTT 5.0 (In Progress)
    - [ ] TLS/TCP
    - [ ] IPV6
- [ ] All MQTT Packet Support (In Progress)
- [ ] Implement `Display` for `Property`
- [ ] Tests
- [ ] no_std support
- [x] Integrate WASM for easy compiling to Javascript/Typescript/Node.js environments
- [ ] Easy internal utility for converting -> to string and vice versal (from terminal tool?) - for debugging
- [ ] Samples for easy learning
- [ ] Move bytes length validation/parsing into the trait, and update the trait's secondary properties
- [ ] Topic Filters: (4.7 Topic Names and Topic Filters)
- [ ] Shared Subscription: (4.8.2 Shared Subscriptions)




### Notes to users:
AsyncReadExt + AsyncWriteExt works fine with async-std, and smol runtime users. However users of tokio, would need to add [tokio-util](crates.io/crates/tokio_util) to ensure compatibility

```tokio
tokio-util = { version = "0.7.13", features= ["compat"]}
let stream = TcpStream::connect("example.com:80").await.unwrap();
let stream = stream.compat();
-->>
```


### Credits:
This crate derives heavy inspiration from:
1. 
2. 