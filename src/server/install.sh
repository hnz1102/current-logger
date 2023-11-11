#!/bin/bash

# Temp-Logger can't directly send data to influxDB API since ESP32-C3 has not TSL software stack.
# This agent program is responsible for changing HTTP communication from Temp-Logger to HTTPS 
# communication and passing data to the InfluxDB API. This program is purpose only for a local
# network because it has no security.

# To install ./install.sh in src/server directory.

export CURRENTLOGGERDIR="/var/lib/currentlogger"

sudo mkdir $CURRENTLOGGERDIR
sudo cp influxdb-agent-start.sh $CURRENTLOGGERDIR
sudo chown root $CURRENTLOGGERDIR/influxdb-agent-start.sh
sudo chgrp root $CURRENTLOGGERDIR/influxdb-agent-start.sh
sudo chmod 755 $CURRENTLOGGERDIR/influxdb-agent-start.sh
sudo cp main.js $CURRENTLOGGERDIR
sudo cp influxdb-agent-current.service /lib/systemd/system/.

cd $CURRENTLOGGERDIR
sudo npm install --save @influxdata/influxdb-client

sudo systemctl daemon-reload
sudo systemctl enable influxdb-agent-current.service
sudo systemctl start influxdb-agent-current.service
sudo systemctl status influxdb-agent-current.service

exit 0