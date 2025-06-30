# s3plz-rs

A simple service that statically hosts assets in an s3 compatible format. 

"statically" here means that the assets that are specified in the programs
args are the only assets that the bucket will hold. You cannot push or modify
the assets, you have to stop and restart the program.

## usage

```bash
$ S3PLZ_PORT=8882 s3plz ./assets:/assets &
$ aws s3 cp localhost:8882/assets ./assets-copy
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
way), I create a new container that runs, literally,
`s3plz <my-k8s-manifests>:/path/flux/looks/in`.

Cursed? Probably!
