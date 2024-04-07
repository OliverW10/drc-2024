
mkdir -p ../micro/src/messages/

protoc -I=messages/ --csharp_out="../sim/Assets/Scripts/Messages/" messages/*

mkdir -p ../planner/messages/
cp messages/*.proto ../planner/messages
mkdir -p ../client/messages/
cp messages/*.proto ../client/messages