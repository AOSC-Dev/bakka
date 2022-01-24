#!/bin/bash

function abbsjump {
    if [[ $1 == "cd" && $2 != "--help" && $2 != "-h" ]]; then
    	cd "$(/usr/local/bin/abbsjump cd ${2})"
    elif [[ $1 == "jump" && $2 != "--help" && $2 != "-h" ]]; then
        cd "$(/usr/local/bin/abbsjump jump ${2})"
    else
        /usr/local/bin/abbsjump $@
    fi
}
