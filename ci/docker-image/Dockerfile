FROM cloudfoundry/cflinuxfs2

RUN apt-get install -y \
    jq \
    python-dev \
    python-pip \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

RUN pip install awscli --ignore-installed six

RUN curl --location 'https://api.bintray.com/content/jfrog/jfrog-cli-go/$latest/jfrog-cli-linux-amd64/jfrog?bt_package=jfrog-cli-linux-amd64' > /usr/local/bin/jfrog && \
    chmod +x /usr/local/bin/jfrog

RUN mkdir -p /usr/local/java \
&&  curl --location 'https://java-buildpack.cloudfoundry.org/openjdk-jdk/trusty/x86_64/openjdk-1.8.0_111.tar.gz' | tar xzf - -C /usr/local/java

ENV JAVA_HOME /usr/local/java
