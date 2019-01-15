#!/bin/sh

SELF_PATH=`realpath $0`
SELF_DIR=`dirname $SELF_PATH`
PROG_DIR=$SELF_DIR/../

if [ $# != 7 ]
then
    echo "Usage:" $0 " <slc port> <data port> <board ip> <cfg> <dump file> <session name> <loop>"
    exit
fi



SLC_PORT=$1
DATA_PORT=$2
BPORT=1234  # Fixed port number
BIP=$3
CFG=$4
DUMP_FILE=$5
SESSION_NAME=$6
loop=$7

session_exists=0

if tmux ls 2>/dev/null
then
    for i in `tmux ls|awk -F ':' '{print $1}'`
    do
		if [ $i == $SESSION_NAME ]
		then
			session_exists=1
			#echo "Session exists."
			break
		fi
    done
fi

if [ $session_exists == 0 ]
then
#        echo "Starting session."
	tmux new -d -s $SESSION_NAME
	sleep 0.1
fi

tmux select-pane -t 0
echo "Now starting server."
tmux send-keys "cargo run -q --manifest-path $PROG_DIR/Cargo.toml --bin trend_server --release -- -a 0.0.0.0 -s ${SLC_PORT} -d ${DATA_PORT} -m 8888 -t ${DUMP_FILE}.yaml -b ${DUMP_FILE}.bin -v 1"  C-m
#sleep .5  # Needed on laptop
echo "Now sending message to board."
sleep 0.1

echo "Nloops="$loop
if [ $loop -gt 1 ]
then
  for i in $(seq 1 $loop); do 
    echo "Now sending request" $i ", then sleep 0.1s."
    cargo run -q --manifest-path $PROG_DIR/Cargo.toml --bin send_msg -- $CFG ${BIP}:${BPORT} 8888
    sleep 0.1
  done
else
    echo "Now sending message to board."
    cargo run -q --manifest-path $PROG_DIR/Cargo.toml --bin send_msg -- $CFG ${BIP}:${BPORT} 8888
fi
