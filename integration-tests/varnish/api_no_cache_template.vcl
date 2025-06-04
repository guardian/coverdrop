vcl 4.1;

backend default {
    .host = "{host}";
    .port = "{port}";
}

sub vcl_recv {
    return(pass);
}