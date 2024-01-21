# should pass the path to protoc as first argument
$1 -I=messages/ --cpp_out=../controller/src/messages/ --python_out=../client/messages/ --csharp_out="../sim/Assets/Scripts/Messages/" messages/*