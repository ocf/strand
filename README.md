# strand

A [Zincati](https://coreos.github.io/zincati/) (Fedora CoreOS) reboot lock backend that makes sure the software running on your nodes is actually healthy before releasing the lock. This is useful because it allows you to run stateful workloads (like Ceph) on CoreOS, and take advantage of auto-updates.

## Supported Strategies

All strategies consist of three parts.

1. The "pre-reboot" conditions and actions. The conditions must be met, and the actions must be taken before the node is given the lock and allowed to reboot.
2. The "post-reboot" conditions and actions. The conditions must be met before a certain timeout in order for the node to be considered healthy. The actions will be taken at some point before or after the conditions to do any neccesary cleanup.
3. The "timeout" action. If the conditions in the "post-reboot" stage are not met in time, the timeout action will be taken, and the lock will be permanently locked until a human manually resolves the issue (by design).

Failures may cause actions in any given stage to get run multiple times. They are guaranteed to run at least once. As such, all actions should be idempotent.

The currently supported strategies, each split up into these three parts, are as follows.

- Kubernetes
  - Pre-reboot: drain + cordon the node
  - Post-reboot: wait for `node.status.conditions[type="Ready"].status == "True"`
  - Timeout: no action taken
- Ceph
  - Pre-reboot: wait for cluster_status == Healthy, set `noout` on OSDs that are about to be `down`
  - Post-reboot: wait for OSDs `up`, unset `noout`, wait for cluster_status == Healthy
  - Timeout: unset `noout` (causing data replication)

In the future, it might be a good idea to allow waiting for arbitrary Prometheus metrics as part of a strategy.

## Locking Design

Strand employs a Kubernetes-native locking design. It depends on the [sequential consistency](https://etcd.io/docs/v3.3/learning/api_guarantees/#consistency) guarantees provided by etcd, and works as follows.

To obtain the lock...

1. Get the `Lease` object you've configured. If it doesn't exist, create it.
    - If some other node holds the lease, fail.
    - If you hold the lease, goto 3.
1. If no node holds the lease, update the lease object with your Node ID.
    - If another node raced you and updated it first, the api server will reject the update, since the object version does not match. Fail.
1. Run the post-reboot actions, return success.

## Metrics and Logs

Strand exports Prometheus metrics on `/metrics`, as is customary. You definitely want to hook into these metrics, because it contains an alarm for when a timeout has been hit and so manual human intervention is required.

Logs might be interesting for debugging issues if the alarm gets hit. Strand makes use of the excellent `tracing` crate, which means it might be possible to hook it into opentelemetry in the future.

## Similar Software

Strand is inspired by [poseidon/fleetlock](https://github.com/poseidon/fleetlock). We really like typhoon but it isn't really for running pet Kubernetes clusters (the author suggests blue/greening your entire Kubernetes cluster if you want to update, which is reasonable for some groups). We need to run Ceph, and also we don't have enough hardware to blue/green an entire deployment, so a little more care with locking is neccesary. If you aren't running Ceph, go check out that project!
