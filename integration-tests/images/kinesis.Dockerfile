# Build a kinesalite service
FROM node:18-alpine AS builder

ENV NODE_ENV production

RUN npm install -g kinesalite@3.3.3 

# Configure our Kinesis set up
FROM builder AS configure

WORKDIR /opt/kinesis

EXPOSE 4567

USER root
RUN apk add --no-cache aws-cli

# The AWS cli wants some credentials set even though we're not going to use them 
RUN  aws configure set aws_access_key_id XXXXXXXXXXXXXXXXXXXX --profile build && \
     aws configure set aws_secret_access_key XXXXXXXXXXXXXXXXXXXX --profile build && \
     aws configure set aws_session_token XXXXXXXXXXXXXXXXXXXX --profile build && \
     aws configure set region XXXXXXXXXXXXXXXXXXXX --profile build

RUN (/usr/local/bin/kinesalite --path /opt/kinesis/ &) && \
                                                          \
    aws --endpoint=http://localhost:4567 --profile build  \
        kinesis create-stream                             \
        --stream-name user-messages                       \
        --shard-count 1 &&                                \
                                                          \
    aws --endpoint=http://localhost:4567 --profile build  \
        kinesis create-stream                             \
        --stream-name journalist-messages                 \
        --shard-count 1 &&                                \
                                                          \
    sleep 5 # Wait for the stream to be created 

CMD ["/usr/local/bin/kinesalite", "--path", "/opt/kinesis/"]
