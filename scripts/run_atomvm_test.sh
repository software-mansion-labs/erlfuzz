#!/bin/bash
set -uo pipefail

main() {
    atomvm_timeout_in_s=60

    # Use environment variables and check if they are set
    atomvm_path="${ATOMVM_PATH:-}"
    packbeam_path="${PACKBEAM_PATH:-}"
    avm_lib_path="${AVM_LIB_PATH:-}"

    if [[ -z "$atomvm_path" || -z "$packbeam_path" || -z "$avm_lib_path" ]]; then
        echo "Error: Please ensure ATOMVM_PATH, PACKBEAM_PATH, and AVM_LIB_PATH are all set."
        exit 2
    fi

    if [ "$#" -eq 0 ]; then
        echo "Usage: $0 <path_to_erlang_file>"
        exit 1
    fi

    filename="$1"
    if ! [[ -f "$filename" ]]; then
        echo "Error: File '$filename' does not exist."
        exit 1
    fi

    directory="${filename%/*}"
    beam_filename="${filename%.*}.beam"
    avm_filename="${filename%.*}.avm"
    
    timeout -k 1 ${atomvm_timeout_in_s} erlc -W0 -o "${directory}" "${filename}"
    erlc_result=$?
    if [ ${erlc_result} -eq 0 ]; then
        if [[ -f "$beam_filename" ]]; then
            timeout -k 1 ${atomvm_timeout_in_s} "${packbeam_path}" "${avm_filename}" "${beam_filename}" "${avm_lib_path}"
            timeout -k 1 ${atomvm_timeout_in_s} "${atomvm_path}" "${avm_filename}" 1> /dev/null
            erl_result=$?
            if [ ${erl_result} -eq 0 ] || [ ${erl_result} -eq 1 ]; then
                echo "File ${beam_filename}: completed normally"
                rm "${beam_filename}"
                rm "${avm_filename}"
            elif [[ ${erl_result} -eq 124 ]] || [[ ${erl_result} -eq 137 ]]; then
                echo "File ${beam_filename}: timeout"
                rm "${beam_filename}"
                rm "${avm_filename}"
            else
                echo "INTERESTING: AVM crashed on ${beam_filename}, produced from: ${filename} with error code ${erl_result}!"
                rm "${beam_filename}"
                rm "${avm_filename}"
                exit ${erl_result}
            fi
        else
            echo "Error: Beam file was not created, check erlc output."
            exit 3
        fi
    else
        echo "INTERESTING: erlc either crashed or timed out on ${filename}!"
        exit 42
    fi
    exit 0
}

main "$@"