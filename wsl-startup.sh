#!/bin/bash
# need use visudo to add the config in the end of sudoers
# <USERNAME> ALL=(root) NOPASSWD: /bin/mount --make-rshared /
# for podman
sudo mount --make-rshared /
