FROM paritytech/ci-linux:production

WORKDIR /var/www/dvine_node
COPY . /var/www/dvine_node
EXPOSE 9944