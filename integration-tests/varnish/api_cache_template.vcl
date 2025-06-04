vcl 4.1;

backend default {
    .host = "{host}";
    .port = "{port}";
}

# dead-drop endpoint default cache time is set to 5 minutes 
# which is slowing down key_consensus test. This endpoint 
# doesn't need to be cached. 
sub vcl_recv {
    if (regsub(req.url, "\?.*$", "") == "/v1/journalist/dead-drops") {
        return(pass);
    } 
}
