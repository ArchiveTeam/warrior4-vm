FROM ubuntu:24.04
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
# ubuntu user is already included in newer images
# RUN useradd --user-group --create-home --no-log-init --shell /bin/bash ubuntu && \
#     adduser ubuntu sudo && \
#     echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers
RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers
USER ubuntu
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN gem install --user-install fpm --bindir /home/ubuntu/.local/share/gem/ruby/bin
ENV PATH="/home/ubuntu/.cargo/bin:/home/ubuntu/.local/share/gem/ruby/bin:$PATH"
RUN rustup target add x86_64-unknown-linux-musl
