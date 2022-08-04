ARG base_image=library/ubuntu:bionic
ARG from_image=bellsoft/liberica-openjdk-debian:8

# need this hack because of a docker bug
#  see https://stackoverflow.com/a/63472135/1585136
FROM ${from_image} as java

FROM ${base_image}

RUN apt-get update && apt-get install -y \
    build-essential \
    clang \
    curl \
    ca-certificates \
    jq \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

ENV JAVA_HOME="/usr/lib/jvm/jdk"
ENV PATH="${JAVA_HOME}/bin:${PATH}"
COPY --from=java $JAVA_HOME $JAVA_HOME

ENV CARGO_HOME  /usr/local
ENV RUSTUP_HOME /usr/local

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

RUN curl -fL https://getcli.jfrog.io | sh && \
    mv jfrog /usr/local/bin/
