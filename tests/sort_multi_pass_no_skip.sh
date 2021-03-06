#!/bin/bash

exit_code=0

VERSIONS=(1 2 3 17 18 19)

source functions.sh

file_size=$(ls -l dummy | awk '{ print $5 }')

# generate test data
dd if=/dev/urandom of=dummy bs=$file_size count=1 &>/dev/null

for ver in ${VERSIONS[*]}; do
    for (( i=0; i < 3; i++ )); do
        if   [[ $ver ==  1 ]]; then
            data_shards=$((1 + RANDOM % 128))
            parity_shards=$((1 + RANDOM % 128))
        elif [[ $ver ==  2 ]]; then
            data_shards=$((1 + RANDOM % 128))
            parity_shards=$((1 + RANDOM % 128))
        elif [[ $ver ==  3 ]]; then
            data_shards=$((1 + RANDOM % 128))
            parity_shards=$((1 + RANDOM % 128))
        elif [[ $ver == 17 ]]; then
            data_shards=$((1 + RANDOM % 128))
            parity_shards=$((1 + RANDOM % 128))
        elif [[ $ver == 18 ]]; then
            data_shards=$((1 + RANDOM % 128))
            parity_shards=$((1 + RANDOM % 128))
        else
            data_shards=$((1 + RANDOM % 128))
            parity_shards=$((1 + RANDOM % 128))
        fi

        burst=$((RANDOM % 15))

        container_name=sort_$data_shards\_$parity_shards\_$ver.sbx

        echo -n "Encoding in version $ver, data = $data_shards, parity = $parity_shards"
        output=$(./../blkar encode --json --sbx-version $ver -f dummy $container_name \
                            --uid DEADBEEF0001 \
                            --hash sha1 \
                            --rs-data $data_shards --rs-parity $parity_shards)
        if [[ $(echo $output | jq -r ".error") != null ]]; then
            echo " ==> Invalid JSON"
            exit_code=1
        fi
        if [[ $(echo $output | jq -r ".stats.sbxVersion") == "$ver" ]]; then
            echo -n " ==> Okay"
        else
            echo -n " ==> NOT okay"
            exit_code=1
        fi
        if [[ $(echo $output | jq -r ".stats.hash" | awk '{ print $1 }') == "SHA1" ]]; then
            echo " ==> Okay"
        else
            echo " ==> NOT okay"
            exit_code=1
        fi

        # Create corrupted copies
        echo "Creating corrupted copies"
        cp $container_name $container_name.1
        cp $container_name $container_name.2
        cp $container_name $container_name.3
        cp $container_name $container_name.4
        cp $container_name $container_name.5
        mv $container_name $container_name.6

        corrupt  5000 $container_name.1
        corrupt 10000 $container_name.1
        corrupt 15000 $container_name.1
        corrupt 20000 $container_name.1

        corrupt 10000 $container_name.2
        corrupt 15000 $container_name.2
        corrupt 20000 $container_name.2

        corrupt 15000 $container_name.3
        corrupt 20000 $container_name.3

        corrupt 20000 $container_name.4

        corrupt  5000 $container_name.5
        corrupt 10000 $container_name.5
        corrupt 15000 $container_name.5

        corrupt  5000 $container_name.6
        corrupt 10000 $container_name.6
        corrupt 15000 $container_name.6
        corrupt 20000 $container_name.6

        echo "Sorting container"
        for i in 1 2 3 4 5 6; do
          echo -n "    pass $i"
          output=$(./../blkar sort --json -f --burst $burst --multi-pass-no-skip $container_name.$i sorted_$container_name)
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

        echo -n "Checking sorted container burst error resistance level"
        output=$(./../blkar show --json --guess-burst sorted_$container_name)
        if [[ $(echo $output | jq -r ".error") != null ]]; then
            echo " ==> Invalid JSON"
            exit_code=1
        fi
        burst_shown=$(echo $output | jq -r ".bestGuessForBurstErrorResistanceLevel")
        if [[ (($ver == "1" || $ver == "2" || $ver == "3") && ($burst_shown == "null"))
            || (($ver == "17" || $ver == "18" || $ver == "19") && ($burst_shown == $burst)) ]]; then
            echo " ==> Okay"
        else
            echo " ==> NOT okay"
            exit_code=1
        fi

        output_name=dummy_$data_shards\_$parity_shards

        echo -n "Decoding"
        output=$(./../blkar decode --json -f sorted_$container_name $output_name)
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

        echo -n "Comparing decoded data to original"
        cmp dummy $output_name
        if [[ $? == 0 ]]; then
            echo " ==> Okay"
        else
            echo " ==> NOT okay"
            exit_code=1
        fi
    done
done

for ver in ${VERSIONS[*]}; do
  for (( i=0; i < 3; i++ )); do
    if   [[ $ver ==  1 ]]; then
      block_size=512
      data_shards=$((1 + RANDOM % 128))
      parity_shards=$((1 + RANDOM % 128))
    elif [[ $ver ==  2 ]]; then
      block_size=128
      data_shards=$((1 + RANDOM % 128))
      parity_shards=$((1 + RANDOM % 128))
    elif [[ $ver ==  3 ]]; then
      block_size=4096
      data_shards=$((1 + RANDOM % 128))
      parity_shards=$((1 + RANDOM % 128))
    elif [[ $ver == 17 ]]; then
      block_size=512
      data_shards=$((1 + RANDOM % 128))
      parity_shards=$((1 + RANDOM % 128))
    elif [[ $ver == 18 ]]; then
      block_size=128
      data_shards=$((1 + RANDOM % 128))
      parity_shards=$((1 + RANDOM % 128))
    else
      block_size=4096
      data_shards=$((1 + RANDOM % 128))
      parity_shards=$((1 + RANDOM % 128))
    fi

    burst=$((RANDOM % 15))

    container_name=sort_$data_shards\_$parity_shards\_$ver.sbx

    echo -n "Encoding in version $ver, data = $data_shards, parity = $parity_shards"
    output=$(./../blkar encode --json --sbx-version $ver -f dummy $container_name.1 \
                        --uid DEADBEEF0001 \
                        --hash sha1 \
                        --rs-data $data_shards --rs-parity $parity_shards \
                        --burst $burst)
    if [[ $(echo $output | jq -r ".error") != null ]]; then
      echo " ==> Invalid JSON"
      exit_code=1
    fi
    if [[ $(echo $output | jq -r ".stats.sbxVersion") == "$ver" ]]; then
      echo -n " ==> Okay"
    else
      echo -n " ==> NOT okay"
      exit_code=1
    fi
    if [[ $(echo $output | jq -r ".stats.hash" | awk '{ print $1 }') == "SHA1" ]]; then
      echo -n " ==> Okay"
    else
      echo -n " ==> NOT okay"
      exit_code=1
    fi

    output=$(./../blkar encode --json --sbx-version $ver -f dummy $container_name.2 \
                        --uid DEADBEEF0002 \
                        --hash sha1 \
                        --rs-data $data_shards --rs-parity $parity_shards \
                        --burst $burst)
    if [[ $(echo $output | jq -r ".error") != null ]]; then
      echo " ==> Invalid JSON"
      exit_code=1
    fi
    if [[ $(echo $output | jq -r ".stats.sbxVersion") == "$ver" ]]; then
      echo -n " ==> Okay"
    else
      echo -n " ==> NOT okay"
      exit_code=1
    fi
    if [[ $(echo $output | jq -r ".stats.hash" | awk '{ print $1 }') == "SHA1" ]]; then
      echo " ==> Okay"
    else
      echo " ==> NOT okay"
      exit_code=1
    fi

    echo -n "Sorting container using 2nd container"
    output=$(./../blkar sort --json -f --burst $burst --multi-pass-no-skip $container_name.2 $container_name.1)
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

    echo -n "Checking sorted container burst error resistance level"
    output=$(./../blkar show --json --guess-burst $container_name.1)
    if [[ $(echo $output | jq -r ".error") != null ]]; then
      echo " ==> Invalid JSON"
      exit_code=1
    fi
    burst_shown=$(echo $output | jq -r ".bestGuessForBurstErrorResistanceLevel")
    if [[ (($ver == "1" || $ver == "2" || $ver == "3") && ($burst_shown == "null"))
              || (($ver == "17" || $ver == "18" || $ver == "19") && ($burst_shown == $burst)) ]]; then
      echo " ==> Okay"
    else
      echo " ==> NOT okay"
      exit_code=1
    fi

    output_name=dummy_$data_shards\_$parity_shards

    echo -n "Decoding"
    output=$(./../blkar decode --json -f $container_name.1 $output_name)
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

    echo -n "Checking container block source"
    cmp $container_name.1 $container_name.2
    if [[ $? == 0 ]]; then
      echo " ==> Okay"
    else
      echo " ==> NOT okay"
      exit_code=1
    fi

    echo -n "Comparing decoded data to original"
    cmp dummy $output_name
    if [[ $? == 0 ]]; then
      echo " ==> Okay"
    else
      echo " ==> NOT okay"
      exit_code=1
    fi
  done
done

echo $exit_code > exit_code
