#!/usr/bin/env bash
# Strict mode: http://redsymbol.net/articles/unofficial-bash-strict-mode/
set -euo pipefail
IFS=$'\n\t'

wget https://github.com/SimonKagstrom/kcov/archive/master.tar.gz 
tar xzf master.tar.gz 
cd kcov-master 
mkdir build 
cd build 
cmake .. 
make 
make install DESTDIR=../../kcov-build 
cd ../.. 
rm -rf kcov-master 
for file in target/debug/laminar-*[^\.d]; do mkdir -p "target/cov/$(basename $file)"; ./kcov-build/usr/local/bin/kcov --exclude-pattern=/.cargo,/usr/lib --verify "target/cov/$(basename $file)" "$file"; done 
bash <(curl -s https://codecov.io/bash)
echo "Uploaded code coverage"
