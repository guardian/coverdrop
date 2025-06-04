# On-premises Deployment

> Note that this document presents a more high level overview of our on-premises architecture
> for a more detailed step-by-step guide to setting up the infra please refer to the [README](../infra/README.md)

For our on-premises stack we cannot use the standard [Riff-Raff](https://github.com/guardian/riff-raff) based deployment strategy that the Guardian uses for it's cloud services.

We have opted to go for a (relatively) modern GitOps type approach which allows us to use open source tooling.

## How the servers are set up

Our on-premises servers run a pretty standard Ubuntu distro with hardening measures applied. On these we run the k3s Kubernetes distribution.

## Kubernetes "layers"

We break down our system into a few layers.

### System

This collection of resources handles the underlying infrastructure required for running the services.
Controllers include ArgoCD, Longhorn, sealed secrets, and facilities to copy files onto service volumes.

### On-premises foundation

This is a very small layer containing the GitOps requirements: the on-premises namespace and ArgoCD configuration.

### On-premises application

This layer containers the rest of the application resources. In production, this layer is managed by ArgoCD.

## Deploying to production

The system and foundation layer are manually applied during the initial set up of a production cluster. The application layers are applied using ArgoCD.
When a new commit is merged into main our continuous integration will create a new PR in the private platform repository which must be manually approved by a
developer on the CoverDrop team. Once that is approved it will be applied by ArgoCD automatically.

To access the ArgoCD UI to see deployment details you must have a machine with appropriate `kubectl` credentials to port forward the web service.

```shell
kubectl port-forward svc/argocd-server -n argocd 8080:443
```

From there the ArgoCD users and passwords can be used.
