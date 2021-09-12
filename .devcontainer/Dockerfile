# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.191.1/containers/rust/.devcontainer/base.Dockerfile

FROM mcr.microsoft.com/vscode/devcontainers/rust:0-1

# [Optional] Uncomment this section to install additional packages.
# RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
#     && apt-get -y install --no-install-recommends <your-package-list-here>

RUN echo "[source.crates-io]\n\
replace-with = 'rsproxy'\n\
[source.rsproxy]\n\
registry = 'https://rsproxy.cn/crates.io-index'\n\
[registries.rsproxy]\n\
index = 'https://rsproxy.cn/crates.io-index'\n\
[net]\n\
git-fetch-with-cli = true\n" > /usr/local/cargo/config
