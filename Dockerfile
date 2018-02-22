FROM rust:1.23-jessie

RUN apt-get update \
    && apt-get install -y libev4 libssl1.0.0 cmake build-essential \
    && wget https://github.com/couchbase/libcouchbase/releases/download/2.8.1/libcouchbase-2.8.1_jessie_amd64.tar \
    && tar xf libcouchbase-2.8.1_jessie_amd64.tar \
    && dpkg -i libcouchbase-2.8.1_jessie_amd64/*.deb

WORKDIR /usr/src/app
COPY . .

RUN cargo install

ENV LOG_LEVEL debug
ENV PORT 6767

CMD jackie -l $LOG_LEVEL -p $PORT
