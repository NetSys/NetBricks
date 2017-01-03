FROM ubuntu:zesty
MAINTAINER "Aurojit Panda <apanda@cs.berkeley.edu>"
RUN apt-get -yq update && apt-get -yq install apt-utils
RUN apt-get -yq update && apt-get -yq install build-essential \
				vim-nox curl \
				pciutils sudo git \
				python python3
RUN apt-get -yq update && apt-get -yq install libssl-dev \
					libgnutls30 libgnutls-openssl-dev \
					libcurl4-gnutls-dev cmake bash libpcap-dev libnuma-dev
RUN mkdir -p ~/.ssh && ssh-keyscan -t rsa github.com > ~/.ssh/known_hosts
ENV SYSTEM_CARGO=1
RUN git clone https://github.com/apanda/cargo.git /opt/cargo
RUN cd /opt/cargo && git submodule update --init
ENV PATH /root/.cargo/bin:$PATH
RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly -y
RUN cd /opt/cargo && /opt/cargo/configure --prefix=/usr/local
RUN cd /opt/cargo && make all -j
RUN cd /opt/cargo && sudo make install -j
RUN cargo install rustfmt
CMD /bin/bash
