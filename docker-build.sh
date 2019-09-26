#!/bin/sh
docker build -f ./Dockerfile-base -t tr_logic_llc/base .
docker-compose up
