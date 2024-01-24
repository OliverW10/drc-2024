# should pass the path to protoc as first argument (e.g. ~/Downloads/protoc/bin/protoc)

if [[ $# -ne 1 ]]; then
    echo "Usage ./build_proto [path to protoc e.g. ~/Downloads/protoc/bin/protoc]"
    exit 1
fi

mkdir -p ../micro/src/messages/
mkdir -p ../client/messages/
mkdir -p ../sim/Assets/Scripts/Messages/
mkdir -p ../micro-py/messages/

$1 -I=messages/ --python_out=../micro-py/messages/ --csharp_out="../sim/Assets/Scripts/Messages/" messages/*