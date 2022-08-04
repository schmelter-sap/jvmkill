ARG base_image=openjdk:8-jre
FROM ${base_image}

RUN apt-get update && apt-get install -y \
    build-essential \
    clang \
    curl \
    ca-certificates \
    jq \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

ENV CARGO_HOME  /usr/local
ENV RUSTUP_HOME /usr/local

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

RUN curl -fL https://getcli.jfrog.io | sh && \
    mv jfrog /usr/local/bin/
