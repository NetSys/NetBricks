FROM debian
MAINTAINER "Aurojit Panda <apanda@cs.berkeley.edu>"
COPY sources.list /etc/apt/sources.list 
RUN apt-get -yq update && apt-get -yq install apt-utils
RUN apt-get -yq update && apt-get -yq install build-essential \
				vim-nox curl \
				pciutils sudo git linux-headers-`uname -r` \
				python
RUN apt-get -yq update && apt-get -yq install libssl-dev \
					libgnutls30 libgnutls-openssl-dev \
					libcurl4-gnutls-dev cmake
# Fix the date at which we take Rust
RUN curl -sSf https://static.rust-lang.org/rustup.sh | sh -s -- \
		--channel=nightly --date=2016-04-13
RUN mkdir -p ~/.ssh && ssh-keyscan -t rsa github.com > ~/.ssh/known_hosts
RUN mkdir -p /opt && git clone https://github.com/NetSys/e2d2.git /opt/e2d2
ENV RTE_SDK=/opt/e2d2/3rdparty/dpdk
ENV RTE_TARGET=build
ENV RTE_ARCH=x86_64
ENV ZCSI_ROOT=/opt/e2d2
ENV LD_LIBRARY_PATH="/opt/e2d2/native:/opt/e2d2/3rdparty/dpdk/build/lib"
ENV DELAY_TEST_ROOT="/opt/e2d2/test/delay-test/target/release"
RUN /opt/e2d2/build.sh
COPY run-script.sh /opt/run-script.sh 
COPY run-script.py /opt/run-script.py
CMD /opt/run-script.py
