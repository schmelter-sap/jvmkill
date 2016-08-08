FROM java:8-jdk

RUN apt-get update && apt-get install -y \
    build-essential \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

RUN curl --location 'https://api.bintray.com/content/jfrog/jfrog-cli-go/$latest/jfrog-cli-linux-amd64/jfrog?bt_package=jfrog-cli-linux-amd64' > /usr/local/bin/jfrog && \
    chmod +x /usr/local/bin/jfrog

RUN apk add --update \
    bash \
    coreutils \
    curl \
    jq \
    py-pip \
    unzip \
 && rm -rf /var/cache/apk/*

RUN pip install awscli --ignore-installed six
