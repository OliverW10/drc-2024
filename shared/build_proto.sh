
mkdir -p ../planner/messages/
cp messages/*.proto ../planner/messages
mkdir -p ../client/messages/
cp messages/*.proto ../client/messages
mkdir -p ../scripts/messages/
protoc -I=messages/ --python_out="../scripts/messages/" messages/*