# Quick start:

## Start Miner 1
```cargo r --  127.0.0.2 127.0.0.1 -p test.usr```

## Start Miner 2
```cargo r --  127.0.0.1 127.0.0.2 -v TRACE  -p test.usr --threads 10```

## Create Client transaction (in  working)
```cargo r --  127.0.0.1 127.0.0.4 -v TRACE  -p test.usr -a 1 -d 1```



# ROAD MAP  

dans la transa si il y a rien en input marquer Miner transaction 

# BUG 

bail!("missing key for unlocking utxo => {}", utxo);  est triger quand la clef priver n'est pas bonne (cargo run -- 127.0.0.2 127.0.0.4  -p other.usr -a 10 -d default.usr)
