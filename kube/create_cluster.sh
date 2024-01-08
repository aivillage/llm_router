#!/bin/bash

gcloud config set project hoth-410100 
export PROJECT_ID=$(gcloud config get project)
export REGION=asia-east1

gcloud container clusters create hoth-demo --location ${REGION} \
  --workload-pool ${PROJECT_ID}.svc.id.goog \
  --enable-image-streaming \
  --node-locations=$REGION-a \
  --workload-pool=${PROJECT_ID}.svc.id.goog \
  --addons GcsFuseCsiDriver   \
  --machine-type n2d-standard-4 \
  --num-nodes 1 --min-nodes 1 --max-nodes 5 \
  --ephemeral-storage-local-ssd=count=2


gcloud container node-pools create p100-test --cluster hoth-demo \
  --accelerator type=nvidia-tesla-p100,count=1,gpu-driver-version=latest \
  --machine-type n1-standard-8 \
  --ephemeral-storage-local-ssd=count=1 \
  --enable-autoscaling --enable-image-streaming \
  --num-nodes=0 --min-nodes=0 --max-nodes=20 \
  --node-locations $REGION-a,$REGION-c --region $REGION --spot