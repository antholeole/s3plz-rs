# s3plz-rs

A simple service that statically hosts assets in an s3 compatible format. 

"statically" means that the assets that are specified in the programs args are
the only assets that the bucket will hold. You cannot push or modify the assets,
you have to stop and restart the program.

builds using nix or cargo.

## usage

```bash
$ S3PLZ_PORT=8882 S3PLZ_HOST=127.0.0.1 s3plz ./s3plz-test-bucket &
$ aws s3 cp --endpoint-url http://localhost:8882 --no-sign-request  s3://s3plz-test-bucket/deployment.yaml ./copy.yaml
download: s3://s3plz-test-bucket/deployment.yaml to ./copy.yaml
```

## Why?

The fact that this project does not already exists means to me that I am doing
something terribly, terribly wrong :)

tl;dr: I want to host my kubernetes manifests so fluxcd can read them!

In my [home server](http://github.com/antholeole/home-server), I have a nix
derivation that builds all the manifests I intend to deploy to my kubernetes
cluster. Because I am SRE-brained, I want to do this through `fluxcd`. Because
I am not SRE brained, I don't want to make a git commit every time I want to
iterate on the home server: instead, I want to be able to nixos rebuild.

So, on every nixos rebuild, if the manifests change (in a nix way, not a git
way), I create a new container that runs, literally, `s3plz <my-k8s-manifests>`.

Cursed? Probably!

## Spec Compliance

I am _NOT_ going for full spec compliance. As features are required, I'll
be happy to implement them _as long as they don't compromise the simplicity of
the api_.

Noteably, you would have to spend a long time convicing me that auth
is worth implementing.

Currently Implemented:
- list v2
- get
- head /
