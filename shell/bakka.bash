#!/bin/bash

function bakka {
    if [[ $1 == "cd" && $2 != -* ]]; then
    	cd "$(command bakka cd ${2})"
    elif [[ $1 == "jump" && $2 != -* ]]; then
        cd "$(command bakka jump ${2})"
    else
        command bakka $@
    fi
}
