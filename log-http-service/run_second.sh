#!/bin/bash
THIS_DIR=$(readlink -f $(dirname $0))
#echo $THIS_DIR

DATA_DIR=$THIS_DIR/app_data/queue-second
SRC_DATA_DIR=$THIS_DIR/app_data/queue

echo "rebuild data dir"

rm -rf $DATA_DIR
cp -a $SRC_DATA_DIR $DATA_DIR

echo "current data:"
du -h $DATA_DIR

EXE=$(readlink -f $THIS_DIR/../target/debug/log-http-service)
echo "exe $EXE"

$EXE -conf $THIS_DIR/dloghw-second.json -wd $THIS_DIR