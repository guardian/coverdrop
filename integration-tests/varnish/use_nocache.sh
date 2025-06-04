/bin/bash
-c
varnishadm vcl.load nocache /etc/varnish/no-cache.vcl && varnishadm vcl.use nocache