#!/bin/bash

function abbsjump {
    A=$1
    B=$2
    if [[ $A == "cd" && $B != "--help" && $B != "-h" ]]; then
    	cd "$(/home/saki/abbsjump/target/debug/abbsjump cd ${B})"
    else
        /home/saki/abbsjump/target/debug/abbsjump $@
    fi
}
