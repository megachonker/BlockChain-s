## Launch Node  

cargo run <ip_bootstart> <ip_node>  
Run one node with ip bind : *ip_node* and try to connect to the network throws *ip_bootstrap* (0.0.0.0 if it is the first node)
```

 

## ROAD MAP


- Fix Transaction : when a new block is found all of the transactions learn by node need to be clear (beacause they risk to add transaction in more than one block )  
- Make rust test
- Impl security (number of Node=public key, verify the signature when someone send transaction) 
- Impl the variation of the dificulty of the proof of work when block is found to fast/slow.

