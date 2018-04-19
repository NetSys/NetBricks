FROM williamofockham/sandbox

LABEL author="Aurojit Panda <apanda@cs.berkeley.edu>"
LABEL maintainer="williamofockham <zeeshan.lakhani@gmail.com>"
RUN apt-get update && \
    apt-get -y --no-install-recommends install \
    libsctp-dev

ENV NETBRICKS_ROOT=/opt/netbricks
ENV LD_LIBRARY_PATH="/opt/netbricks/native:/opt/netbricks/3rdparty/dpdk/build/lib"
ENV DELAY_TEST_ROOT="/opt/netbricks/test/delay-test/target/release"
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
COPY . /opt/netbricks
WORKDIR /opt/netbricks
RUN git clean -dffx && make compile
CMD /bin/bash
