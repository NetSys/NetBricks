FROM ubuntu:artful
MAINTAINER "Aurojit Panda <apanda@cs.berkeley.edu>"
RUN apt-get -yq update && apt-get -yq install apt-utils
RUN apt-get -yq update && apt-get -yq install build-essential \
				vim-nox curl \
				pciutils sudo git \
				python python3 gosu
RUN apt-get -yq update && apt-get -yq install libssl-dev \
                libsctp-dev libgnutls30 libgnutls-openssl-dev \
                libcurl4-gnutls-dev cmake bash libpcap-dev libnuma-dev \
                tcpdump clang-5.0
RUN mkdir -p ~/.ssh && ssh-keyscan -t rsa github.com > ~/.ssh/known_hosts
RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly -y
ENV PATH /root/.cargo/bin:$PATH
RUN rustup component add rustfmt-preview --toolchain=nightly
RUN rustup update
# RUN cargo install rustfmt-nightly || true
CMD [/bin/bash]
