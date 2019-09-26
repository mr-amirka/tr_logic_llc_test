#!/bin/sh
curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly -y
apt-get update -y && apt-get install -y cmake wget

curl -sL https://deb.nodesource.com/setup_12.x | bash -
apt-get install -y nodejs

# Установка утилит
npm i -g mnpkg

# Установка зависимостей для OpenCV
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/universe/libs/libsoxr/libsoxr0_0.1.2-1_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/main/o/orc/liborc-0.4-0_0.4.25-1_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/main/g/glibc/multiarch-support_2.23-0ubuntu11_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/universe/x/x265/libx265-79_1.9-3_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/universe/x/x264/libx264-148_0.148.2643+git5c65704-1_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/main/libw/libwebp/libwebp5_0.4.4-1_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/main/libv/libvpx/libvpx3_1.5.0-2ubuntu1_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/universe/libv/libva/libva1_1.7.0-1ubuntu0.1_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/universe/s/schroedinger/libschroedinger-1.0-0_1.0.11-2.1build1_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/main/libp/libpng/libpng12-0_1.2.54-1ubuntu1.1_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/universe/o/openjpeg/libopenjpeg5_1.5.2-3.1_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/universe/f/ffmpeg/libavutil-ffmpeg54_2.8.15-0ubuntu0.16.04.1_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/universe/f/ffmpeg/libswresample-ffmpeg1_2.8.15-0ubuntu0.16.04.1_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/universe/c/crystalhd/libcrystalhd3_0.0~git20110715.fdd2f19-11build1_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/universe/libg/libgsm/libgsm1_1.0.13-4_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/universe/l/lame/libmp3lame0_3.99.5+repack1-9build1_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/main/o/opus/libopus0_1.1.2-1ubuntu1_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/universe/s/shine/libshine3_3.1.0-4_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/main/s/snappy/libsnappy1v5_1.1.3-2_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/main/s/speex/libspeex1_1.2~rc1.2-1ubuntu1_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/main/libo/libogg/libogg0_1.3.2-1_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/main/libt/libtheora/libtheora0_1.1.1+dfsg.1-8_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/universe/t/twolame/libtwolame0_0.3.13-1.2_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/main/libv/libvorbis/libvorbis0a_1.3.5-3ubuntu0.2_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/main/libv/libvorbis/libvorbisenc2_1.3.5-3ubuntu0.2_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/main/w/wavpack/libwavpack1_4.75.2-2ubuntu0.2_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/universe/x/xvidcore/libxvidcore4_1.3.4-1_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/universe/z/zvbi/libzvbi-common_0.2.35-10_all.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/universe/z/zvbi/libzvbi0_0.2.35-10_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/universe/f/ffmpeg/libavcodec-ffmpeg56_2.8.6-1ubuntu2_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/universe/libb/libbluray/libbluray1_0.9.2-2_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/universe/libm/libmodplug/libmodplug1_0.8.8.5-2_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/universe/g/game-music-emu/libgme0_0.6.0-3ubuntu0.16.04.1_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/main/libs/libssh/libssh-gcrypt-4_0.6.3-4.3ubuntu0.2_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/universe/f/ffmpeg/libavcodec-ffmpeg-extra56_2.8.15-0ubuntu0.16.04.1_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/universe/f/ffmpeg/libavformat-ffmpeg56_2.8.15-0ubuntu0.16.04.1_amd64.deb
mnpkg -l http://archive.ubuntu.com/ubuntu/pool/universe/f/ffmpeg/libswscale-ffmpeg3_2.8.15-0ubuntu0.16.04.1_amd64.deb

# Скачивание исходников OpenCV 4.1.1
mkdir ./ffi
cd ./ffi && git clone https://github.com/opencv/opencv.git
cd ./opencv && git checkout ddbd10c0019f3ee5f43b7902d47e7fc4303a6574
cd ..

# Компиляция OpenCV
mkdir ./build && cd ./build
cmake -D CMAKE_BUILD_TYPE=Release -D OPENCV_GENERATE_PKGCONFIG=YES -D CMAKE_INSTALL_PREFIX=/usr/local ../opencv
make
make install
make install cmake_force
ldconfig
cd ../..
rm -rf ./ffi
export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig/
export LD_LIBRARY_PATH=/usr/local/lib/
