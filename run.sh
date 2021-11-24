#!/bin/bash

function kill_cctl {
  killall cctl
}

trap kill_cctl INT

trap kill_cctl TERM

trap kill_cctl HUP

cd "$(dirname "$0")"

./target/release/cctl &

cd web

pnpm preview -- --host 0.0.0.0 --port 3005
