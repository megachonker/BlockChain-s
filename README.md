# Quick start:

## Create a wallet (cryptographic keys)  

```
cargo run --  -c   && mv default.usr bob.usr
```  

It is create a wallet ```bob.usr``` with the keys for the blockchaine.  

## Start a miner  

```cargo run -- 0.0.0.0 127.0.0.1 -p bob.usr```

It launch a miner (with 1 thread, you can modify with --thread) miner mine for the wallet bob.usr

At this time the blockchain is running (in localhost). You can launch another miner with another wallet.

```cargo run --  127.0.0.1 127.0.0.2 -p alice.usr```


In argument the first IP is the IP of the server want to connect (here the addresse of the first miner) and the second IP is the IP of the miner.

To see statistic of a wallet you can run 
```cargo run -- 127.0.0.1 127.0.0.3 -p bob.usr -s ```

127.0.0.3 is the ip of the client which run this command


Finally to make a trasaction between alice and bob you can run  

```cargo run -- 127.0.0.1 127.0.0.3 -p bob.usr -a 10 -d alice.usr```




For more information check ```--help option```



