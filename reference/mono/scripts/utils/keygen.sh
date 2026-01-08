#!/bin/bash

usage() {
    echo "Usage: $0 [-l key_length] [-f format]"
    echo "Generate a cryptographically secure random key"
    echo
    echo "Options:"
    echo "  -l    Key length in bits (128, 192, 256, 384, 512)"
    echo "  -f    Output format (hex, base64, base64url)"
    echo "        default: hex"
    echo
    echo "Example:"
    echo "  $0 -l 256 -f base64"
    exit 1
}

# Get byte length from bits
get_bytes() {
    local bits=$1
    case $bits in
        128) echo 16 ;; # AES-128
        192) echo 24 ;; # AES-192
        256) echo 32 ;; # AES-256
        384) echo 48 ;; # SHA-384
        512) echo 64 ;; # SHA-512
        *) echo "Error: Invalid key length. Valid options: 128, 192, 256, 384, 512" >&2; exit 1 ;;
    esac
}

# Default values
KEY_LENGTH=256
FORMAT="hex"

# Parse command line arguments
while getopts "l:f:h" opt; do
    case $opt in
        l)
            KEY_LENGTH=$OPTARG
            ;;
        f)
            FORMAT=$OPTARG
            ;;
        h)
            usage
            ;;
        \?)
            usage
            ;;
    esac
done

# Get number of bytes needed
BYTES=$(get_bytes $KEY_LENGTH)

# Generate random bytes using /dev/urandom
RANDOM_BYTES=$(dd if=/dev/urandom bs=1 count=$BYTES 2>/dev/null)

# Output in requested format
case $FORMAT in
    "hex")
        echo "$RANDOM_BYTES" | xxd -p | tr -d '\n'
        ;;
    "base64")
        echo "$RANDOM_BYTES" | base64
        ;;
    "base64url")
        echo "$RANDOM_BYTES" | base64 | tr '+/' '-_' | tr -d '='
        ;;
    *)
        echo "Error: Invalid format. Valid options: hex, base64, base64url" >&2
        exit 1
        ;;
esac