FROM debian
MAINTAINER "Aurojit Panda <apanda@cs.berkeley.edu>"
COPY sources.list /etc/apt/sources.list 
RUN apt-get -yq update && apt-get -yq install apt-utils
RUN apt-get -yq update && apt-get -yq install build-essential \
				vim-nox curl \
				pciutils sudo git linux-headers-`uname -r`
RUN apt-get -yq update && apt-get -yq install libssl-dev \
					libgnutls30 libgnutls-openssl-dev \
					libcurl4-gnutls-dev cmake
RUN curl -sSf https://static.rust-lang.org/rustup.sh | sh -s -- --channel=nightly
RUN mkdir -p ~/.ssh && ssh-keyscan -t rsa github.com > ~/.ssh/known_hosts
RUN mkdir -p /opt && git clone https://github.com/NetSys/e2d2.git /opt/e2d2
ENV RTE_SDK=/opt/e2d2/3rdparty/dpdk
ENV RTE_TARGET=build
ENV RTE_ARCH=x86_64
RUN /opt/e2d2/build.sh
COPY run-script.sh /opt/run-script.sh 
CMD /opt/run-script.sh
