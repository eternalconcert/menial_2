#!/bin/bash

OS_INFO=$(uname -mrs)
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
BUILDNO=$1
VERSION=$2

INDEXFILE=default/welcomepage/index.html

cp deployment/index.neutral $INDEXFILE
sed -i -- 's/{{ OS_INFO }}/'"${OS_INFO}"'/g' $INDEXFILE
sed -i -- 's/{{ TIMESTAMP }}/'"${TIMESTAMP}"'/g' $INDEXFILE
sed -i -- 's/{{ BUILDNO }}/'"${BUILDNO}"'/g' $INDEXFILE
sed -i -- 's/{{ VERSION }}/'"${VERSION}"'/g' $INDEXFILE
