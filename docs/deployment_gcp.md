# GCP Deployment (Dev)
[Google Help](https://cloud.google.com/artifact-registry/docs/docker/pushing-and-pulling)
Make sure to run docker and gcloud both as sudo, or the same user otherwise they reference different config files and things get confusing (don't ask me how long that took to debug)

~~Disclaimer: I did this the day before so I could have missed a step~~
I did end up missing a step but this should be everything now

```bash
#either sign in here
gcloud init
#or sign in with this
gcloud auth login

#configure docker
#Remember that this could update the wrong config if you there's a user mismatch
gcloud auth configure-docker us-east1-docker.pkg.dev


#build the docker image
docker build -t us-east1-docker.pkg.dev/mempools-387914/mempools-repo/mempools-server-dev:stag --file ./deployments/dockerfiles/dev/Dockerfile .
#or for prod:stag
docker build -t us-east1-docker.pkg.dev/mempools-387914/mempools-repo/mempools-server-prod:stag --file ./deployments/dockerfiles/prod/Dockerfile .

#push the image(dev)
docker push us-east1-docker.pkg.dev/mempools-387914/mempools-repo/mempools-server-dev:stag
```