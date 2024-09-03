FROM ubuntu:22.04
RUN apt-get update && \
    apt-get -y install \
    sudo \
    build-essential \
    musl-tools \
    qemu-utils \
    curl \
    wget \
    rsync \
    fdisk \
    e2fsprogs \
    ruby-full \
    && apt-get clean
RUN useradd --user-group --create-home --no-log-init --shell /bin/bash ubuntu && \
    adduser ubuntu sudo && \
    echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers
USER ubuntu
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN gem install --user-install fpm
ENV PATH="/home/ubuntu/.cargo/bin:/home/ubuntu/.local/share/gem/ruby/3.0.0/bin:$PATH"
RUN rustup target add x86_64-unknown-linux-musl
