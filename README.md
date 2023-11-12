# Quick start:

## Start Miner 1
```cargo r --  127.0.0.2 127.0.0.1 -p test.usr```

## Start Miner 2
```cargo r --  127.0.0.1 127.0.0.2 -v TRACE  -p test.usr --threads 10```

## Create Client transaction (in  working)
```cargo r --  127.0.0.1 127.0.0.4 -v TRACE  -p test.usr -a 1 -d 1```
