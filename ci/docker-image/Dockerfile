FROM java:8-jdk

RUN apt-get update && apt-get install -y \
    gcc-4.7 \
    build-essential \
    jq \
    python-pip \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

RUN pip install awscli --ignore-installed six

RUN curl --location 'https://api.bintray.com/content/jfrog/jfrog-cli-go/$latest/jfrog-cli-linux-amd64/jfrog?bt_package=jfrog-cli-linux-amd64' > /usr/local/bin/jfrog && \
    chmod +x /usr/local/bin/jfrog
