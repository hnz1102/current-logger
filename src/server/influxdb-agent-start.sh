#!/bin/bash

export NODE_HOME=/var/lib/currentlogger
export PATH=/usr/bin:$PATH

/usr/bin/node $NODE_HOME/main.js &

exit 0
