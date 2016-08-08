FROM java:8-jdk

RUN apt-get update && apt-get install -y \
    build-essential \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

RUN apt-get install python \
 && curl -O https://bootstrap.pypa.io/get-pip.py \
 && python get-pip.py \
 && pip install awscli --ignore-installed six \
 && apt-get install jq

RUN curl --location 'https://api.bintray.com/content/jfrog/jfrog-cli-go/$latest/jfrog-cli-linux-amd64/jfrog?bt_package=jfrog-cli-linux-amd64' > /usr/local/bin/jfrog && \
    chmod +x /usr/local/bin/jfrog
