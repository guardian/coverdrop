# CoverUp

Tool for administering a CoverDrop on-premises cluster.

## Why not `admin`?

The `admin` tool is intended to operate on the "application" layer of CoverDrop,
e.g to create keys, create vaults, run ceremonies, etc, but has no particular
knowledge of the specific infrastructural makeup of the on-premises cluster.

As such, while the `admin` tool would be useful to anyone who is supporting a CoverDrop
system, `coverup` is useful to someone who is running CoverDrop in specifically the
same way that the Guardian does.

Some of the commands in `coverup` utilise the `admin` tool as a library.

Unless you have a very good reason we recommend that you use the infrastructural setup
that we have developed for use at the Guardian. That way you can benefit directly from
many of the decisions we have made around security, reliability and maintainability.
