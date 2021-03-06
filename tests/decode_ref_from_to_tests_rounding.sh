#!/bin/bash

exit_code=0

source functions.sh

offset=$[1 + RANDOM % 100]

echo -n "Encoding"
output=$(./../blkar encode --json -f dummy --uid DEADBEEF0001 --sbx-version 17 --rs-data 3 --rs-parity 2)
if [[ $(echo $output | jq -r ".error") != null ]]; then
    echo " ==> Invalid JSON"
    exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.fileUID") == "DEADBEEF0001" ]]; then
    echo " ==> Okay"
else
    echo " ==> NOT okay"
    exit_code=1
fi

echo -n "Decoding dummy disk"

output=$(./../blkar decode -f --json --ref-from $offset dummy.ecsbx)
if [[ $(echo $output | jq -r ".error") == "null" ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi

corrupt 0 dummy.ecsbx

output=$(./../blkar decode -f --json --ref-from $offset --ref-to-inc $offset dummy.ecsbx)
if [[ $(echo $output | jq -r ".error") == "Error : Failed to find reference block" ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi

output=$(./../blkar decode -f --json --ref-from $[offset + 512] dummy.ecsbx)
if [[ $(echo $output | jq -r ".error") == "null" ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo $exit_code > exit_code
