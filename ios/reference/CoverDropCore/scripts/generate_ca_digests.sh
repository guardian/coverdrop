#!/usr/bin/env bash
set -e

cat <<EOF
This script will generate digests for CA root certificates

This is required to do certificate pinning in the reference app

secure-messaging.code.dev-guardianapis.com uses Global Sign
../Sources/CoverDropCore/Resources/public_ca_keys/trusted_root_global_sign.pem

coverdrop-api.code.dev-gutools.co.uk uses Amazon CA
../Sources/CoverDropCore/Resources/public_ca_keys/trusted_root_amazon.pem

If you need to update these certificate digests (if a new root CA is published)
update ../../reference/Info.plist with the new digests

<key>NSAppTransportSecurity</key>
    <dict>
        <key>NSPinnedDomains</key>
        <dict>
            <key>secure-messaging.code.dev-guardianapis.com</key>
            <dict>
                <key>NSPinnedCAIdentities</key>
                <array>
                    <dict>
                        <key>SPKI-SHA256-BASE64</key>
update --->     			<string>cGuxAXyFXFkWm61cF4HPWX8S0srS9j0aSqN0k4AP+4A=</string>
                    </dict>
                </array>
            </dict>
            <key>coverdrop-api.code.dev-gutools.co.uk</key>
            <dict>
                <key>NSPinnedCAIdentities</key>
                <array>
                    <dict>
                        <key>SPKI-SHA256-BASE64</key>
update --->     			<string>++MBgDH5WGvL9Bcn5Be30cRcL0f5O+NyoXuWtQdX1aI=</string>
                    </dict>
                </array>
            </dict>
        </dict>
    </dict>
EOF

get_digest () {
    CERT_PATH=$1
    DOMAIN=$2
    STACK=$3
    DIGEST=$(cat $CERT_PATH | 
    openssl x509 -inform pem -noout -outform pem -pubkey | 
    openssl pkey -pubin -inform pem -outform der | 
    openssl dgst -sha256 -binary | 
    openssl enc -base64)
    echo "-----Digest for $STACK------"
    echo "Digest: $DIGEST"
    echo "Domain: $DOMAIN" 
    echo "Certificate path: $CERT_PATH"
    echo "-----------------"
}

get_digest ../Sources/CoverDropCore/Resources/public_ca_keys/trusted_root_amazon.pem secure-messaging.code.dev-guardianapis.com code
get_digest ../Sources/CoverDropCore/Resources/public_ca_keys/trusted_root_global_sign.pem coverdrop-api.code.dev-gutools.co.uk code

get_digest ../Sources/CoverDropCore/Resources/public_ca_keys/trusted_root_amazon.pem secure-messaging-api-audit.guardianapis.com audit
get_digest ../Sources/CoverDropCore/Resources/public_ca_keys/trusted_root_amazon.pem secure-messaging-msg-audit.guardianapis.com audit