#!/usr/bin/env sh
set -e
set -u
set -x

testBranch=$1;
concurrency=$2;

gitrepo="$(git rev-parse --show-toplevel)"

masterdir=/tmp/rmace-master
newdir="/tmp/rmace-$testBranch"

rm -rf "$masterdir"
rm -rf "$newdir"

git clone "$gitrepo" "$masterdir"
git clone "$gitrepo" "$newdir"

pushd "$masterdir"
git checkout master
popd

pushd "$newdir"
git checkout "$testBranch"
popd

for dir in "$masterdir" "$newdir"; do
    pushd "$dir"
    cargo build --release --bin uci
    popd
done

cutechess-cli \
    -engine name="rmace-$testBranch" proto=uci  cmd="$newdir"/target/release/uci \
    -engine name=rmace-master proto=uci cmd="$masterdir"/target/release/uci \
    -openings file="$masterdir"/etc/8moves_v3.pgn \
    -games 2 -rounds 2500 -repeat 2 -maxmoves 200 \
    -sprt elo0=0 elo1=10 alpha=0.05 beta=0.05 \
    -concurrency $concurrency \
    -ratinginterval 10 \
    -each  tc=inf/10+0.1
