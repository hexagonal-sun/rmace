#!/usr/bin/env sh
set -e
set -u
set -x

testBranch=$1;

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
    cargo build --release --bin uci
done

cutechess-cli \
    -engine name=mace-new proto=uci  cmd="$newdir"/target/release/uci \
    -engine name=mace-master proto=uci cmd="$masterdir"/target/release/uci \
    -openings file="$masterdir"/etc/silversuite.pgn \
    -concurrency 3 -ratinginterval 10 -games 500 -pgnout games.pgn \
    -each  tc=5+0.01 -repeat
