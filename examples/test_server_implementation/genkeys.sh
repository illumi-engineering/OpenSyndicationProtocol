#!/usr/bin/env bash

mkdir -p ./keys

echo "Generating keys"
ssh-keygen -t rsa -b 4096 -N "" -C "server test key" -f keys/server_id_rsa
ssh-keygen -t rsa -b 4096 -N "" -C "client test key" -f keys/client_id_rsa

echo "Converting keys to RSA format (from OpenSSH)"
ssh-keygen -p -f keys/server_id_rsa -m pem
ssh-keygen -p -f keys/client_id_rsa -m pem