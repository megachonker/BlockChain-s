## How to run the program ?  

You need the run a first Node with the following arguments :

-s **number**    *--->To setup the number of the node*  
-p **ip:port** *--->To pass the ip and the port of the Node*

For the next Node your must setup the same arguments and pass the ip:port of another node of the blockchain :  

-g **ip:port** 


### For example to launch 3 nodes :

```
cargo run -- -s 1 -p 127.0.0.1:6060 
cargo run -- -s 2 -p 127.0.0.2:6060 -g 127.0.0.1:6060
cargo run -- -s 3 -p 127.0.0.3:6060 -g 127.0.0.1:6060
```


To send transactions you can use the same program and setup this following arguments :  

-s **number** *--->the Node which will be send coins*  
-r **number** *--->the Node wich will be recived coins*  
-c **amount** *--->fix the number of coin to sent*  
-m send *--->To setup the mode send, default the mode is set to mine*  
-p **ip:port** *--->setup the ip:port of the sender*  
-g **ip:port** *--->setup the ip:port for send the transaction  


for example

```
cargo run -- -s 0 -r 1 -c 1 -m send -p 127.0.0.2:6060 -g 127.0.0.1:6060 
```



## ROAD MAP


- Fix Transaction : when a new block is found all of the transactions learn by node need to be clear (beacause they risk to add transaction in more than one block )  
- Impl security (number of Node=public key, verify the signature when someone send transaction) 
- Impl the variation of the dificulty of the proof of work when block is found to fast/slow.

