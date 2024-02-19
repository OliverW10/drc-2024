
pushd ./shared
./build_proto.sh
popd

pushd ./planner
cargo build
popd

# build micro and deploy to pico