#!/bin/bash

gnome-terminal -- bash -c "cd ../edgeless/target/debug && RUST_LOG=info ./edgeless_con_d; exec bash" & 	# Start EDGELESS controller
gnome-terminal -- bash -c "cd ../edgeless/target/debug && RUST_LOG=info ./edgeless_orc_d; exec bash" & 	# Start EDGELESS orchestrator
