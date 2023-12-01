# Ordinals indexer with channel feature

This is an example of how to use the Ordinals indexer with the channel feature. Run against signet with:

```
cargo run  -- --signet --data-dir tmp/.ord --bitcoin-data-dir <your btc node data dir> --rpc-url <your btc node rpc>
```

A cookie file may need to be created and specified if your node is using authentication. You can use the `--cookie-file` option to specify the path to the cookie file. The cookie file should be in the format `__cookie__:<token>`.

```
cargo run  -- --signet --data-dir tmp/.ord --bitcoin-data-dir <your btc node data dir> --rpc-url <your btc node rpc> --cookie-file <path to cookie file>
```
