#!/bin/bash

source kcov_blkar_fun.sh

exit_code=0

VERSIONS=(1)

# Encode in all 3 versions
for ver in ${VERSIONS[*]}; do
  echo -n "Encoding in version $ver"
  output=$(blkar encode --json --sbx-version $ver -f --no-meta dummy dummy$ver.sbx)
  if [[ $(echo $output | jq -r ".error") != null ]]; then
      echo " ==> Invalid JSON"
      exit_code=1
  fi
  if [[ $(echo $output | jq -r ".stats.sbxVersion") == "$ver" ]]; then
      echo " ==> Okay"
  else
      echo " ==> NOT okay"
      exit_code=1
  fi
done

# Check all of them
for ver in ${VERSIONS[*]}; do
  echo -n "Checking version $ver container"
  output=$(blkar check --json --verbose dummy$ver.sbx)
  if [[ $(echo $output | jq -r ".error") != null ]]; then
      echo " ==> Invalid JSON"
      exit_code=1
  fi
  if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedCheck") == 0 ]]; then
      echo " ==> Okay"
  else
      echo " ==> NOT okay"
      exit_code=1
  fi
done

# Decode all of them
for ver in ${VERSIONS[*]}; do
  echo -n "Decoding version $ver container"
  output=$(blkar decode --json -f dummy$ver.sbx dummy$ver)
  if [[ $(echo $output | jq -r ".error") != null ]]; then
      echo " ==> Invalid JSON"
      exit_code=1
  fi
  if [[ $(echo $output | jq -r ".stats.sbxVersion") == "$ver" ]]; then
      echo " ==> Okay"
  else
      echo " ==> NOT okay"
      exit_code=1
  fi
done

# Compare to original file
for ver in ${VERSIONS[*]}; do
  echo -n "Comparing decoded version $ver container data to original"
  cmp dummy dummy$ver &>/dev/null
  if [[ $? == 0 ]]; then
    echo "==> NOT okay"
    exit_code=1
  else
    echo "==> Okay"
  fi
done

echo $exit_code > exit_code
