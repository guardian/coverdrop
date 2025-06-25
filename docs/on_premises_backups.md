## On-premises file system backups

We backup the longhorn volumes used by the CoverNode and Identity API using longhorn's [Backup and Restore](https://longhorn.io/docs/1.9.0/snapshots-and-backups/backup-and-restore/) features.

In order to avoid sending backups outside of the local network, the backup target is a fourth on-premises node, outside of the kubernetes cluster. In dev this fourth node is a multipass instance and in Staging it is an EC2 instance in a dedicated autoscaling group.

The `install_minio` Ansible playbook downloads minio server and configure it in [single-node, multi-drive mode](https://min.io/docs/minio/linux/operations/install-deploy-manage/deploy-minio-single-node-multi-drive.html#minio-snmd), creates a bucket for longhorn backups, and creates a user which longhorn will use to access the bucket.

A manifest file longhorn-backup.yaml creates the backup target in the longhorn-system namespace, and creates a recurring job to create a backup of all volumes on a fixed schedule.
