#!/bin/bash
fd . c.mnt -H -t f -x python scripts.meta/config_linker.py --no-confirm-deletion {}
