#!/bin/bash

if [ $# -ne 1 ]; then
    echo "Illegal number of parameters : $0 number_node"
    exit
fi

if [ "$1" -lt 1 ]; then
    echo "number of node must be biger than 1"
    exit
fi

if [ "$1" -gt 255 ]; then
    echo "number of node must be smaller than 127"
    exit
fi





pid=()






node=1
end_value=$1
last="0.0.0.0"
while [ $node -ne $(($1 + 1)) ] 
do
    ip="127.0.0.$node"

    echo "run $ip on bootstrap $last"
    cargo run $last $ip &

    pid+=($!)


    last=$ip
    node=$(($node + 1))

done

trap custom_interrupt SIGINT

custom_interrupt() {
    echo ${pid[@]}
    for p in ${pid[@]}
    do
        echo "stop $p"
        kill $p 
    done

    exit

}

while true
do
    sleep 1
done




