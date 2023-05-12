#!/bin/bash

RunAOT=false

MeasureMem=false

MeasurePerf=false

BenchRoot="$HOME/ccs_open_source/benches"

BenchSize=5
BenchSuite=()
# Structure:  Benchmark directory                             Native           NativeArg         Iter  WasmDir
BenchSuite+=("tsf"                               "./tsf"            "10000"           "1"    ".")
BenchSuite+=("cjpeg"                  "./cjpeg"      \
             "-dct int -progressive -opt -outfile output_large_encode.jpeg input_large.ppm" "1" ".")
BenchSuite+=("djpeg"                  "./djpeg"      \
             "-dct int -ppm -outfile output_large_decode.ppm input_large.jpg" "1" ".")
BenchSuite+=("bzip2"                     "./bzip2"          "-k -f -z input_file"                     "1"   ".")
BenchSuite+=("espeak"                    "./espeak"         "-f input.txt -s 120 -w output_file.wav"  "1"   ".")
BenchSuite+=("facedetection"             "./facedetection"  "input.png"                               "1"   ".")
BenchSuite+=("gnuchess"                  "./gnuchess"       "< input"                                 "1"   ".")
BenchSuite+=("whitedb"                   "./whitedb"        ""                                        "1"   ".")
BenchSuite+=("rg"                        "./rg"             "'123'"                                   "1"   ".")
BenchSuite+=("coreutils"                 "./coreutils"      "fmt < coreutils.wat"                     "1"   ".")

NumBench=$( echo "scale=0; ${#BenchSuite[@]} / $BenchSize" | bc -l )

runScript() {
    Wasm=$Native.wasm
    WasmAOT=$Native.cwasm

    Wasmtime="$HOME/ccs_open_source/wasmtime/target/release/wasmtime"

    runaot() {
        cmd="$1"
        if [ "$2" = "-n" ] # dry run
        then 
            echo $1
            echo ""
            return 0
        fi
        start=`date +%s.%N`
        sh -c "$cmd"
        end=`date +%s.%N`
        aottime=$( echo "$end - $start" | bc -l )
        echo "AOT compilation time: $aottime seconds"
    }

    runtest() {
        cmd="$1 >$2 2>&1"
        if [ "$4" = "-n" ] # dry run
        then
            echo $cmd
            echo ""
            return 0
        fi
        if [ "$MeasureMem" = true ]
        then
            sh -c "/usr/bin/time -v $cmd"
            mem=$( cat "$2" | grep "Maximum resident set size (kbytes)" | sed 's/.*: //' )
            echo -e "$3:   \t$mem kbytes"
        elif [ "$MeasurePerf" = true ]
        then
    : '
            sh -c "perf stat $cmd"
            cycles=$( cat "$2" | grep "cycles" | sed 's/      cycles.*//' | sed 's/        //' )
            insns=$( cat "$2" | grep "instructions" | sed 's/      instructions.*//' | sed 's/        //')
            branches=$( cat "$2" | grep "branches" | grep -v "branch-misses" | sed 's/      branches.*//' | sed 's/        //')
            brmisses=$( cat "$2" | grep "branch-misses" | sed 's/      branch-misses.*//' | sed 's/        //')
            echo -e "$3:   \t$cycles cycles"
            echo -e "$3:   \t$insns instructions"
            echo -e "$3:   \t$branches branches"
            echo -e "$3:   \t$brmisses branch-misses"
    '
            sh -c "perf stat -e cache-misses,cache-references $cmd"
            cachemisses=$( cat "$2" | grep "cache-misses" | sed 's/      cache-misses.*//' | sed 's/        //' )
            cacherefs=$( cat "$2" | grep "cache-references" | sed 's/      cache-references.*//' | sed 's/        //')
            echo -e "$3:   \t$cachemisses cache-misses"
            echo -e "$3:   \t$cacherefs cache-references"
        else
            start=`date +%s.%N`
            for (( i=1; i<=$Iter; i++ ))
            do
                echo $cmd
                sh -c "$cmd"
            done
            end=`date +%s.%N`
            runtime=$( echo "$end - $start" | bc -l )
            itertime=$( echo "scale=11; $runtime / $Iter" | bc -l )
            if [ "${itertime::1}" = "." ]
            then
                itertime="0${itertime}"
            fi
            # echo -e "$3:   \t$itertime seconds" >> output_time
            #echo "Total run time: $runtime seconds"
            #echo "Each iter time: $itertime seconds"
            #cat "$2"
        fi
        if grep -q "ERROR\|Error\|error\|Exception\|exception\|Fail\|fail" "$2"
        then
            echo "Error encountered. Please double-check"
        fi
    }

    if [ ! -z "$WasmDir" ]
    then
        WasmtimeDir="--dir $WasmDir"
    fi

    if [ ! -z "$NativeArg" ]
    then
        WasmtimeNativeArg="-- $NativeArg"
    fi

    #echo "Iteration(s): $Iter"

    #: '
    #echo ""

    if [ "$RunAOT" = true ]
    then
    runaot "$Wasmtime compile $Wasm -o $WasmAOT" $1
    runtest "$Wasmtime run --allow-precompiled $WasmtimeDir $WasmAOT $WasmtimeNativeArg" "output_wasmtime" "wasmtime" $1
    else
    runtest "$Wasmtime run $WasmtimeDir $Wasm $WasmtimeNativeArg" "output_wasmtime" "wasmtime" $1
    fi

    #echo ""

    if [ "$1" == "-n" ] # No need to compare results for a dry run
    then
        return 0
    fi
}

for (( idx=0; idx<${#BenchSuite[@]}; idx+=${BenchSize} ));
do
    nth=$( echo "scale=0; $idx / $BenchSize" | bc -l)
    nth=$((nth+1))

    # For debugging
    #if [ "$nth" -ne 4 ]
    #then
    #    continue
    #fi

    echo "[${nth}/${NumBench}] ${BenchSuite[idx]}"

    # Enter benchmark directory
    # cd ${BenchSuite[idx]}

    # Setup environment 
    Native=${BenchSuite[idx+1]}
    NativeArg=${BenchSuite[idx+2]}
    Iter=$( echo ${BenchSuite[idx+3]} | bc -l )
    WasmDir=${BenchSuite[idx+4]}

    # Run benchmark
    # echo "Running..."
    runScript

    # Enter the root directory
    cd "$BenchRoot"
    echo ""
done