#!/bin/sh
apt-get install -y gcc
aptitude -f install -y libcurl4-openssl-dev \
    libelf-dev \
    libdw-dev \
    binutils-dev \
    libiberty-dev \
  && rm -rf /var/lib/apt/lists/* \
  && wget https://github.com/SimonKagstrom/kcov/archive/master.tar.gz \
  && apt-get remove --yes --purge wget \
  && tar xzf master.tar.gz \
  && cd kcov-master \
  && mkdir build \
  && cd build \
  && cmake -DCMAKE_BUILD_TYPE=Release .. \
  && make \
  && make install \
  && cd ../.. \
  && rm -rf kcov-master
