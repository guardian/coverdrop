## Generating SPKI-SHA256-BASE64 for Certificate Authority pinning

The SPKI-SHA256-BASE64 were generated using :

```
cat <cert_path> | \
openssl x509 -inform pem -noout -outform pem -pubkey | \
openssl pkey -pubin -inform pem -outform der | \
openssl dgst -sha256 -binary | \
openssl enc -base64
```

by using the 2 root certs in the android code base

```
android/app/src/main/res/raw/trusted_root_global_sign.pem
android/app/src/main/res/raw/trusted_root_amazon.pem
```

`android/app/src/main/res/raw/trusted_root_global_sign.pem` is used for `code.dev-guardianapis.com`
`android/app/src/main/res/raw/trusted_root_amazon.pem` is used for `code.dev-gutools.co.uk`

These are added in Info.plist as pinned CA's

See https://developer.apple.com/news/?id=g9ejcf8y for more details
