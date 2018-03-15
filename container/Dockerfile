FROM debian:testing
LABEL maintainer="Aurojit Panda <apanda@cs.berkeley.edu>"
LABEL maintainer="williamofockham <zeeshan.lakhani@gmail.com>"
ARG dpdk_file="common_linuxapp-17.08.container"
RUN apt-get -yq update && apt-get -yq install apt-utils
RUN apt-get -yq update && apt-get -yq install build-essential \
				vim-nox curl \
				pciutils sudo git \
				python python3
RUN apt-get -yq update && apt-get -yq install libssl-dev \
					libgnutls30 libgnutls-openssl-dev libsctp-dev \
					libcurl4-gnutls-dev cmake bash libpcap-dev libnuma-dev \
					clang-5.0 libclang-dev
# Fix the date at which we take Rust
RUN mkdir -p ~/.ssh && ssh-keyscan -t rsa github.com > ~/.ssh/known_hosts
ENV RTE_SDK=/opt/netbricks/3rdparty/dpdk
ENV RTE_TARGET=build
ENV RTE_ARCH=x86_64
ENV NETBRICKS_ROOT=/opt/netbricks
ENV DPDK_CONFIG_FILE="/opt/netbricks/3rdparty/dpdk-confs/$dpdk_file"
ENV LD_LIBRARY_PATH="/opt/netbricks/native:/opt/netbricks/3rdparty/dpdk/build/lib"
ENV DELAY_TEST_ROOT="/opt/netbricks/test/delay-test/target/release"
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
RUN mkdir -p /opt/netbricks
COPY Cargo.toml /opt/netbricks
COPY 3rdparty /opt/netbricks/3rdparty
COPY framework /opt/netbricks/framework
COPY native /opt/netbricks/native
COPY patches /opt/netbricks/patches
COPY scripts /opt/netbricks/scripts
COPY test /opt/netbricks/test
COPY .gitignore /opt/netbricks/.gitignore
COPY LICENSE.md /opt/netbricks/LICENSE.md
COPY README.md /opt/netbricks/README.md
COPY build.sh /opt/netbricks/build.sh
COPY examples.sh /opt/netbricks/examples.sh
COPY rustfmt.toml /opt/netbricks/rustfmt.toml
RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly -y
ENV PATH /root/.cargo/bin:$PATH
RUN /opt/netbricks/build.sh
CMD /bin/bash
