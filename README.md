# Image processing lambda function.

## What is this code?

This is part of the [image gallery](https://gallery.eduartepaiva.com) project, the intent of this code is downloading an image from one s3 bucket, This image was uploaded with some userful metadata, then the code, based on the metadata will process the image and re-upload it in another bucket. After that it'll delete the original image from the first bucket.

## What is the objective?
The aim of the lambda function is to optmize heavy sized images, like a 10mb png image, then it'll crop and convert the image to JPEG, it'll compress the image too.

## How to build for deployment on a aws lambda?
I'm using [Cargo Lambda](https://www.cargo-lambda.info/) to create and build this project, my specific cli command is:  
> cargo lambda build --release --arm64 --output-format zip

The arm64 flag make it possible do deploy on arm witch I think it's better.


## Env variables:

BUCKET_TO_GET_IMAGE  
BUCKET_TO_PUT_IMAGE
