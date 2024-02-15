
mkdir -p ../micro/src/messages/
mkdir -p ../client/messages/
mkdir -p ../sim/Assets/Scripts/Messages/

protoc -I=messages/ --csharp_out="../sim/Assets/Scripts/Messages/" messages/*

mkdir -p ../planner/messages/
cp messages/*.proto ../planner/messages