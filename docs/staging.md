# CoverDrop 'STAGING' Environment

At the guardian we set up a staging environment in aws that is more or less a copy of our production environment - with
the same setup steps and cluster configuration for the on-premises services.

## Deploying changes to STAGING

# One off - you only need to do this if the staging stack has been replaced
1. Fetch the staging stack context file: `cargo run --bin coverup staging kubeconfig --aws-profile secure-collaboration --aws-region eu-west-1`

# For every deploy
1. First, push your branch to the coverdrop repo. 
2. Use the github actions UI to manually run a build for the 'build-prod-images' workflow on your branch
3. Once the build has completed, find the generated branch in the coverdrop-platform repo
4. Connect to argo ci: `cargo run --bin coverup staging argo --aws-profile secure-collaboration --aws-region eu-west-1`

_Note: if you have set the AWS_PROFILE and AWS_REGION environment variables then you don't need to provide the extra arguments._

5. Fetch the argo ci admin password from the shared lastpass. (This password may change if the staging stack has been
recently torn down) 
6. Go to `http://localhost:8085` and login with username 'admin' 
7. Click 'sync' - it will popup a panel on the right. Paste the branch name or commit ID into the 'revision' field and click 'sync'
8. Wait for the sync to complete. You can check logs etc either directly in the argo dashboard or at logs.gutools.co.uk

## Accessing other dashboards
Run `cargo run --bin coverup staging -h` to see other dashboards you can access
